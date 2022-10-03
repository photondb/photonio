use std::{
    future::Future,
    io::Result,
    mem,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use slab::Slab;

#[derive(Clone, Default)]
pub(super) struct OpTable(Arc<Mutex<Slab<OpState>>>);

impl OpTable {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn insert(&mut self) -> OpHandle {
        let mut table = self.0.lock().unwrap();
        let index = table.insert(OpState::default());
        OpHandle::new(self.clone(), index)
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
            OpState::Completed(result) => {
                table.remove(index);
                Poll::Ready(result)
            }
        }
    }

    pub(super) fn complete(&mut self, index: usize, result: Result<u32>) {
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
            OpState::Completed(..) => unreachable!(),
        }
    }
}

#[derive(Default)]
enum OpState {
    #[default]
    Init,
    Polled(Waker),
    Completed(Result<u32>),
}

pub(crate) struct OpHandle {
    table: OpTable,
    index: usize,
    is_finished: bool,
}

impl OpHandle {
    fn new(table: OpTable, index: usize) -> Self {
        Self {
            table,
            index,
            is_finished: false,
        }
    }

    pub(super) fn index(&self) -> usize {
        self.index
    }
}

impl Drop for OpHandle {
    fn drop(&mut self) {
        assert!(self.is_finished);
    }
}

impl Future for OpHandle {
    type Output = Result<u32>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let index = self.index;
        self.table.poll(index, cx.waker()).map_ok(|v| {
            self.is_finished = true;
            v
        })
    }
}
