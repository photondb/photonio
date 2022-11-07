use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::FutureExt;
use photonio::task;

struct Inner {
    closed: bool,
    wakers: Vec<Waker>,
}

#[derive(Clone)]
struct Closer {
    inner: Arc<Mutex<Inner>>,
}

impl Closer {
    fn new() -> Self {
        let inner = Inner {
            closed: false,
            wakers: Vec::new(),
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn wait(&self) {
        while self.inner.lock().unwrap().wakers.is_empty() {}
    }

    fn close(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.closed = true;
        for waker in inner.wakers.drain(..) {
            waker.wake();
        }
    }
}

impl Future for Closer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            Poll::Ready(())
        } else {
            inner.wakers.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct Waiter<F> {
    futures: Vec<F>,
}

impl<F: Future> Waiter<F> {
    fn new() -> Self {
        Self {
            futures: Vec::new(),
        }
    }

    fn add(&mut self, future: F) {
        self.futures.push(future);
    }
}

impl<F: Future + Unpin> Future for Waiter<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for fut in &mut self.futures {
            match fut.poll_unpin(cx) {
                Poll::Ready(v) => return Poll::Ready(v),
                Poll::Pending => {}
            }
        }
        Poll::Pending
    }
}

#[photonio::test]
async fn wake() {
    let close1 = Closer::new();
    let close2 = Closer::new();
    let mut waiter = Waiter::new();
    waiter.add(close1.clone());
    waiter.add(close2.clone());
    let handle = task::spawn(waiter);
    close1.wait();
    close2.wait();
    close1.close();
    close2.close();
    handle.await.unwrap();
}
