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
pub(super) struct RawTask {
    id: u64,
    vtable: &'static VTable,
}

impl RawTask {
    pub fn new<F, S>(id: u64, future: F, schedule: S) -> NonNull<RawTask>
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        let suit = Arc::new(Suit::new(id, future, schedule));
        unsafe { Self::from_suit(suit) }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub unsafe fn drop(&self, this: NonNull<RawTask>) {
        (self.vtable.drop)(this)
    }

    pub unsafe fn clone(&self, this: NonNull<RawTask>) -> NonNull<RawTask> {
        (self.vtable.clone)(this)
    }

    pub unsafe fn poll(&self, this: NonNull<RawTask>) {
        (self.vtable.poll)(this)
    }

    pub unsafe fn join<T>(&self, this: NonNull<RawTask>, waker: &Waker) -> Poll<T> {
        let mut result = Poll::Pending;
        (self.vtable.join)(this, waker, &mut result as *mut _ as *mut _);
        result
    }

    pub unsafe fn detach(&self, this: NonNull<RawTask>) {
        (self.vtable.detach)(this)
    }
}

impl RawTask {
    unsafe fn from_suit<F, S>(handle: Arc<Suit<F, S>>) -> NonNull<RawTask>
    where
        F: Future,
        S: Schedule,
    {
        let ptr = Arc::into_raw(handle);
        NonNull::new_unchecked(ptr as *mut RawTask)
    }

    unsafe fn into_suit<F, S>(this: NonNull<RawTask>) -> Arc<Suit<F, S>>
    where
        F: Future,
        S: Schedule,
    {
        Arc::from_raw(this.as_ptr() as *mut _)
    }
}

#[repr(C)]
struct Suit<F, S>
where
    F: Future,
    S: Schedule,
{
    head: RawTask,
    // TODO: optimize this lock
    core: Mutex<Core<F, S>>,
}

impl<F, S> Suit<F, S>
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    fn new(id: u64, future: F, schedule: S) -> Self {
        Self {
            head: RawTask {
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
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = unsafe { RawTask::from_suit(arc_self.clone()) };
        let core = arc_self.core.lock().unwrap();
        core.schedule.schedule(Task(task));
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

enum State<F: Future> {
    Init,
    Detached,
    Finished(F::Output),
    Consumed,
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

struct VTable {
    drop: unsafe fn(NonNull<RawTask>),
    clone: unsafe fn(NonNull<RawTask>) -> NonNull<RawTask>,
    poll: unsafe fn(NonNull<RawTask>),
    join: unsafe fn(NonNull<RawTask>, &Waker, *mut ()),
    detach: unsafe fn(NonNull<RawTask>),
}

impl VTable {
    fn new<F, S>() -> &'static VTable
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        &VTable {
            drop: drop::<F, S>,
            clone: clone::<F, S>,
            poll: poll::<F, S>,
            join: join::<F, S>,
            detach: detach::<F, S>,
        }
    }
}

unsafe fn suit<F, S>(raw: NonNull<RawTask>) -> ManuallyDrop<Arc<Suit<F, S>>>
where
    F: Future,
    S: Schedule,
{
    ManuallyDrop::new(RawTask::into_suit(raw))
}

unsafe fn drop<F, S>(raw: NonNull<RawTask>)
where
    F: Future,
    S: Schedule,
{
    let mut this = suit::<F, S>(raw);
    ManuallyDrop::drop(&mut this);
}

unsafe fn clone<F, S>(raw: NonNull<RawTask>) -> NonNull<RawTask>
where
    F: Future,
    S: Schedule,
{
    let this = suit::<F, S>(raw);
    let clone = ManuallyDrop::into_inner(this).clone();
    RawTask::from_suit(clone)
}

unsafe fn poll<F, S>(raw: NonNull<RawTask>)
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let this = suit::<F, S>(raw);
    let waker = waker_ref(&this);
    let mut cx = Context::from_waker(&waker);
    let mut core = this.core.lock().unwrap();
    let future = Pin::new_unchecked(&mut core.future);
    if let Poll::Ready(output) = future.poll(&mut cx) {
        core.finish(output);
    }
}

unsafe fn join<F, S>(raw: NonNull<RawTask>, waker: &Waker, result: *mut ())
where
    F: Future,
    S: Schedule,
{
    let this = suit::<F, S>(raw);
    let mut core = this.core.lock().unwrap();
    *(result as *mut Poll<_>) = core.join(waker);
}

unsafe fn detach<F, S>(raw: NonNull<RawTask>)
where
    F: Future,
    S: Schedule,
{
    let this = suit::<F, S>(raw);
    let mut core = this.core.lock().unwrap();
    core.detach();
}
