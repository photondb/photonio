use std::{
    future::Future,
    mem::ManuallyDrop,
    panic,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::task::{waker_ref, ArcWake};

use super::{Result, Schedule, Task};

#[repr(C)]
pub(super) struct Head {
    id: u64,
    vtable: &'static VTable,
}

impl Head {
    pub(super) fn id(&self) -> u64 {
        self.id
    }

    pub(super) unsafe fn drop(&self, this: &Arc<Head>) {
        (self.vtable.drop)(this);
    }

    pub(super) unsafe fn poll(&self, this: &Arc<Head>) {
        (self.vtable.poll)(this);
    }

    pub(super) unsafe fn join<T>(&self, this: &Arc<Head>, waker: &Waker) -> Poll<Result<T>> {
        let mut result = Poll::Pending;
        (self.vtable.join)(this, waker, &mut result as *mut _ as *mut _);
        result
    }

    pub(super) unsafe fn detach(&self, this: &Arc<Head>) {
        (self.vtable.detach)(this);
    }
}

#[repr(C)]
pub(super) struct Suit<F, S>
where
    F: Future,
    S: Schedule,
{
    head: Head,
    core: Mutex<Core<F, S>>,
}

impl<F, S> Suit<F, S>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
    S: Schedule + Send,
{
    pub(super) fn new(id: u64, future: F, schedule: S) -> Self {
        Self {
            head: Head {
                id,
                vtable: VTable::new::<F, S>(),
            },
            core: Mutex::new(Core {
                state: State::Init,
                waker: None,
                future,
                schedule,
            }),
        }
    }
}

impl<F, S> ArcWake for Suit<F, S>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
    S: Schedule + Send,
{
    fn wake_by_ref(this: &Arc<Self>) {
        let task = Task::from_suit(this.clone());
        let core = this.core.lock().unwrap();
        core.schedule.schedule(task);
    }
}

struct Core<F, S>
where
    F: Future,
    S: Schedule,
{
    state: State<F::Output>,
    waker: Option<Waker>,
    future: F,
    schedule: S,
}

enum State<T> {
    Init,
    Detached,
    Finished(Result<T>),
    Consumed,
}

impl<F, S> Core<F, S>
where
    F: Future,
    S: Schedule,
{
    fn join(&mut self, waker: &Waker) -> Poll<Result<F::Output>> {
        match std::mem::replace(&mut self.state, State::Init) {
            State::Init => {
                self.waker = Some(waker.clone());
                Poll::Pending
            }
            State::Finished(result) => {
                self.state = State::Consumed;
                Poll::Ready(result)
            }
            _ => unreachable!(),
        }
    }

    fn detach(&mut self) {
        match std::mem::replace(&mut self.state, State::Init) {
            State::Init => {
                self.state = State::Detached;
            }
            State::Detached => unreachable!(),
            State::Finished(_) => {
                self.state = State::Consumed;
            }
            State::Consumed => {}
        }
    }

    fn finish(&mut self, result: Result<F::Output>) {
        match std::mem::replace(&mut self.state, State::Init) {
            State::Init => {
                self.state = State::Finished(result);
                if let Some(waker) = self.waker.take() {
                    waker.wake();
                }
            }
            State::Detached => {
                self.state = State::Consumed;
            }
            State::Finished(_) | State::Consumed => unreachable!(),
        }
    }
}

struct VTable {
    drop: unsafe fn(&Arc<Head>),
    poll: unsafe fn(&Arc<Head>),
    join: unsafe fn(&Arc<Head>, &Waker, *mut ()),
    detach: unsafe fn(&Arc<Head>),
}

impl VTable {
    fn new<F, S>() -> &'static VTable
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
        S: Schedule + Send,
    {
        &VTable {
            drop: drop::<F, S>,
            poll: poll::<F, S>,
            join: join::<F, S>,
            detach: detach::<F, S>,
        }
    }
}

unsafe fn suit<F, S>(head: &Arc<Head>) -> Arc<Suit<F, S>>
where
    F: Future,
    S: Schedule,
{
    Arc::from_raw(Arc::as_ptr(head) as _)
}

unsafe fn drop<F, S>(head: &Arc<Head>)
where
    F: Future,
    S: Schedule,
{
    suit::<F, S>(head);
}

unsafe fn poll<F, S>(head: &Arc<Head>)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
    S: Schedule + Send,
{
    let suit = ManuallyDrop::new(suit::<F, S>(head));
    let waker = waker_ref(&suit);
    let mut cx = Context::from_waker(&waker);
    let mut core = suit.core.lock().unwrap();
    let future = Pin::new_unchecked(&mut core.future);
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| future.poll(&mut cx)));
    match result {
        Ok(Poll::Pending) => {}
        Ok(Poll::Ready(output)) => core.finish(Ok(output)),
        Err(err) => core.finish(Err(err)),
    }
}

unsafe fn join<F, S>(head: &Arc<Head>, waker: &Waker, result: *mut ())
where
    F: Future,
    S: Schedule,
{
    let suit = ManuallyDrop::new(suit::<F, S>(head));
    let mut core = suit.core.lock().unwrap();
    *(result as *mut Poll<_>) = core.join(waker);
}

unsafe fn detach<F, S>(head: &Arc<Head>)
where
    F: Future,
    S: Schedule,
{
    let suit = ManuallyDrop::new(suit::<F, S>(head));
    let mut core = suit.core.lock().unwrap();
    core.detach();
}
