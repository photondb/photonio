use std::{
    future::Future,
    io::Result,
    mem,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use slab::Slab;

pub(crate) struct Op {
    table: OpTable,
    index: usize,
}

impl Op {
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Drop for Op {
    fn drop(&mut self) {
        self.table.cancel(self.index);
    }
}

impl Future for Op {
    type Output = Result<u32>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let index = self.index;
        self.table.poll(index, cx.waker())
    }
}

#[derive(Default)]
enum OpState {
    #[default]
    Init,
    Polled(Waker),
    Canceled,
    Completed(Result<u32>),
}

#[derive(Clone, Default)]
pub(super) struct OpTable(Arc<Mutex<Slab<OpState>>>);

impl OpTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self) -> Op {
        let mut table = self.0.lock().unwrap();
        let index = table.insert(OpState::Init);
        Op {
            index,
            table: self.clone(),
        }
    }

    fn poll(&mut self, index: usize, waker: &Waker) -> Poll<Result<u32>> {
        let mut table = self.0.lock().unwrap();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init => {
                *state = OpState::Polled(waker.clone());
                Poll::Pending
            }
            OpState::Polled(w) => {
                if !w.will_wake(waker) {
                    *state = OpState::Polled(waker.clone());
                }
                Poll::Pending
            }
            OpState::Canceled => unreachable!(),
            OpState::Completed(result) => {
                table.remove(index);
                Poll::Ready(result)
            }
        }
    }

    pub fn cancel(&mut self, index: usize) {
        let mut table = self.0.lock().unwrap();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init | OpState::Polled(_) => {
                *state = OpState::Canceled;
            }
            OpState::Canceled => unreachable!(),
            OpState::Completed(_) => {
                table.remove(index);
            }
        }
    }

    pub fn complete(&mut self, index: usize, result: Result<u32>) {
        let mut table = self.0.lock().unwrap();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init => {
                *state = OpState::Completed(result);
            }
            OpState::Polled(w) => {
                *state = OpState::Completed(result);
                w.wake();
            }
            OpState::Canceled => {}
            OpState::Completed(..) => unreachable!(),
        }
    }
}
