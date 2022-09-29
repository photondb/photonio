use std::{
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    ptr::NonNull,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::task::{waker_ref, ArcWake};

use super::{Schedule, Task};

#[repr(C)]
pub(super) struct RawTask<F, S>
where
    F: Future,
    S: Schedule,
{
    head: Head,
    core: Mutex<Core<F, S>>,
}

impl<F, S> RawTask<F, S>
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    pub fn new(id: u64, future: F, schedule: S) -> Self {
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

impl<F, S> ArcWake for RawTask<F, S>
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = Task::from_raw(arc_self.clone());
        let core = arc_self.core.lock().unwrap();
        core.schedule.schedule(task);
    }
}

struct Core<F, S>
where
    F: Future,
    S: Schedule,
{
    state: State<F>,
    waker: Option<Waker>,
    future: F,
    schedule: S,
}

impl<F, S> Core<F, S>
where
    F: Future,
    S: Schedule,
{
    fn join(&mut self, waker: &Waker) -> Poll<F::Output> {
        match std::mem::replace(&mut self.state, State::Init) {
            State::Init => {
                self.waker = Some(waker.clone());
                Poll::Pending
            }
            State::Finished(output) => {
                self.state = State::Consumed;
                Poll::Ready(output)
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

    fn finish(&mut self, output: F::Output) {
        match std::mem::replace(&mut self.state, State::Init) {
            State::Init => {
                self.state = State::Finished(output);
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

enum State<F: Future> {
    Init,
    Detached,
    Finished(F::Output),
    Consumed,
}

#[repr(C)]
pub(super) struct Head {
    pub(super) id: u64,
    pub(super) vtable: &'static VTable,
}

pub(super) struct VTable {
    pub(super) poll: unsafe fn(NonNull<Head>),
    pub(super) join: unsafe fn(NonNull<Head>, &Waker, *mut ()),
    pub(super) detach: unsafe fn(NonNull<Head>),
}

impl VTable {
    fn new<F, S>() -> &'static VTable
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        &VTable {
            poll: poll::<F, S>,
            join: join::<F, S>,
            detach: detach::<F, S>,
        }
    }
}

unsafe fn harness<F, S>(head: NonNull<Head>) -> ManuallyDrop<Arc<RawTask<F, S>>>
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    ManuallyDrop::new(Arc::from_raw(head.as_ptr() as *const _))
}

unsafe fn poll<F, S>(head: NonNull<Head>)
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let raw = harness::<F, S>(head);
    let waker = waker_ref(&raw);
    let mut cx = Context::from_waker(&waker);
    let mut core = raw.core.lock().unwrap();
    let future = Pin::new_unchecked(&mut core.future);
    if let Poll::Ready(output) = future.poll(&mut cx) {
        core.finish(output);
    }
}

unsafe fn join<F, S>(head: NonNull<Head>, waker: &Waker, result: *mut ())
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let raw = harness::<F, S>(head);
    let mut core = raw.core.lock().unwrap();
    *(result as *mut Poll<_>) = core.join(waker);
}

unsafe fn detach<F, S>(head: NonNull<Head>)
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let raw = harness::<F, S>(head);
    let mut core = raw.core.lock().unwrap();
    core.detach();
}
