use std::{
    future::Future,
    pin::pin,
    sync::{Arc, Condvar, Mutex},
    task::{Context, Poll},
};

use futures::task::{waker_ref, ArcWake};

struct Parker {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Parker {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn park(&self) {
        let mut unparked = self.mutex.lock().unwrap();
        while !*unparked {
            unparked = self.condvar.wait(unparked).unwrap();
        }
    }

    fn unpark(&self) {
        let mut unparked = self.mutex.lock().unwrap();
        *unparked = true;
        self.condvar.notify_one();
    }
}

impl Default for Parker {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcWake for Parker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.unpark()
    }
}

thread_local!(static CURRENT: Arc<Parker> = Arc::default());

pub(super) fn block_on<F: Future>(future: F) -> F::Output {
    CURRENT.with(|parker| {
        let waker = waker_ref(parker);
        let mut cx = Context::from_waker(&waker);
        let mut future = pin!(future);
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                return output;
            }
            parker.park();
        }
    })
}
