use std::{
    future::Future,
    task::{Poll, Waker},
};

pub struct RawTask {
    id: u64,
}

impl RawTask {
    pub fn new<F>(id: u64, future: F) -> Self
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        todo!()
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn join<T>(&self, waker: &Waker) -> Poll<T> {
        todo!()
    }
}

fn vtable<F>() -> &'static VTable
where
    F: Future,
{
    &VTable { join: join::<F> }
}

struct VTable {
    join: unsafe fn(*const (), *mut (), &Waker),
}

unsafe fn join<F>(task: *const (), data: *mut (), waker: &Waker)
where
    F: Future,
{
    todo!()
}
