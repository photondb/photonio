use std::{
    future::Future,
    ptr::NonNull,
    sync::Arc,
    task::{Poll, Waker},
};

use futures::task::ArcWake;

mod join;
pub use join::JoinHandle;

pub struct Task(NonNull<Head>);

impl Task {
    pub fn new<F>(id: u64, future: F) -> Self
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let core = Arc::new(Core::new(id, future));
        let head = unsafe {
            let ptr = Arc::into_raw(core);
            NonNull::new_unchecked(ptr as *mut Head)
        };
        Self(head)
    }

    fn vtable(&self) -> &'static VTable {
        unsafe { self.0.as_ref().vtable }
    }

    pub fn poll(self) {
        unsafe {
            (self.vtable().poll)(self.0);
        }
    }

    pub fn join<T>(&self, waker: &Waker) -> Poll<T> {
        let mut poll = Poll::Pending;
        unsafe {
            (self.vtable().join)(self.0, waker, &mut poll as *mut _ as *mut _);
        }
        poll
    }
}

#[repr(C)]
pub struct Core<F: Future> {
    head: Head,
    body: Body<F>,
}

#[repr(C)]
pub struct Head {
    id: u64,
    vtable: &'static VTable,
}

struct Body<F: Future> {
    future: F,
}

impl<F> Core<F>
where
    F: Future,
{
    pub fn new(id: u64, future: F) -> Self {
        Self {
            head: Head {
                id,
                vtable: VTable::new::<F>(),
            },
            body: Body { future },
        }
    }
}

impl<F: Future> ArcWake for Core<F> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        todo!()
    }
}

pub struct VTable {
    pub poll: unsafe fn(NonNull<Head>),
    pub join: unsafe fn(NonNull<Head>, &Waker, *mut ()),
}

impl VTable {
    fn new<F>() -> &'static VTable
    where
        F: Future,
    {
        &VTable {
            poll: poll::<F>,
            join: join::<F>,
        }
    }
}

unsafe fn poll<F: Future>(head: NonNull<Head>) {}

unsafe fn join<F: Future>(head: NonNull<Head>, waker: &Waker, result: *mut ()) {
    let core = head.cast::<Core<F>>();
}
