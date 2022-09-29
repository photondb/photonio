//! Types and operations for asynchronous tasks.
//!
//! This module is similar to [`std::thread`], but for asynchronous tasks instead of threads.

use std::{
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    ptr::NonNull,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::task::{waker_ref, ArcWake};

mod join;
pub use join::JoinHandle;

mod schedule;
use schedule::Schedule;

/// A handle to a task.
pub struct Task(NonNull<Head>);

/// A unique identifier for a task.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(u64);

impl Task {
    pub(crate) fn new<F, S>(id: u64, future: F, schedule: S) -> Self
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        let core = Arc::new(Core::new(id, future, schedule));
        Self::with_core(core)
    }

    pub fn id(&self) -> TaskId {
        TaskId(self.head().id)
    }

    pub(crate) fn poll(self) {
        unsafe {
            (self.head().vtable.poll)(self.0);
        }
    }

    pub(crate) fn join<T>(&self, waker: &Waker) -> Poll<T> {
        let mut result = Poll::Pending;
        unsafe {
            (self.head().vtable.join)(self.0, waker, &mut result as *mut _ as *mut _);
        }
        result
    }
}

impl Task {
    fn with_core<F, S>(core: Arc<Core<F, S>>) -> Self
    where
        F: Future,
        S: Schedule,
    {
        let head = unsafe {
            let ptr = Arc::into_raw(core);
            NonNull::new_unchecked(ptr as *mut Head)
        };
        Self(head)
    }

    fn head(&self) -> &Head {
        unsafe { self.0.as_ref() }
    }
}

#[repr(C)]
struct Core<F, S>
where
    F: Future,
    S: Schedule,
{
    head: Head,
    body: Mutex<Body<F, S>>,
}

#[repr(C)]
struct Head {
    id: u64,
    vtable: &'static VTable,
}

struct Body<F, S>
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
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    fn new(id: u64, future: F, schedule: S) -> Self {
        Self {
            head: Head {
                id,
                vtable: VTable::new::<F, S>(),
            },
            body: Mutex::new(Body {
                state: State::Init,
                waker: None,
                future,
                schedule,
            }),
        }
    }
}

impl<F, S> ArcWake for Core<F, S>
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = Task::with_core(arc_self.clone());
        let body = arc_self.body.lock().unwrap();
        body.schedule.schedule(task);
    }
}

struct VTable {
    poll: unsafe fn(NonNull<Head>),
    join: unsafe fn(NonNull<Head>, &Waker, *mut ()),
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
        }
    }
}

unsafe fn poll<F, S>(head: NonNull<Head>)
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let core = ManuallyDrop::new(Arc::from_raw(head.as_ptr() as *const Core<F, S>));
    let waker = waker_ref(&core);
    let mut cx = Context::from_waker(&waker);
    let mut body = core.body.lock().unwrap();
    let future = Pin::new_unchecked(&mut body.future);
    if let Poll::Ready(output) = future.poll(&mut cx) {
        body.state = State::Finished(output);
    }
}

unsafe fn join<F, S>(head: NonNull<Head>, waker: &Waker, result: *mut ())
where
    F: Future + Send,
    F::Output: Send,
    S: Schedule + Send,
{
    let core = ManuallyDrop::new(Arc::from_raw(head.as_ptr() as *const Core<F, S>));
    let mut body = core.body.lock().unwrap();
    let result = result as *mut Poll<F::Output>;
    match std::mem::replace(&mut body.state, State::Init) {
        State::Init => {
            body.waker = Some(waker.clone());
            *result = Poll::Pending;
        }
        State::Finished(output) => {
            body.state = State::Consumed;
            *result = Poll::Ready(output);
        }
        _ => unreachable!(),
    }
}
