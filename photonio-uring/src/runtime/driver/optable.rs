use std::{
    io::Result,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use slab::Slab;

#[derive(Default)]
enum OpState {
    #[default]
    Init,
    Polled(Waker),
    Completed(Result<u32>),
}

#[derive(Clone, Default)]
pub(super) struct OpTable(Arc<Mutex<Slab<OpState>>>);

impl OpTable {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn add(&mut self) -> usize {
        let mut table = self.0.lock().unwrap();
        table.insert(OpState::default())
    }

    pub(super) fn poll(&mut self, index: usize, waker: &Waker) -> Poll<Result<u32>> {
        let mut table = self.0.lock().unwrap();
        let state = table.get_mut(index).unwrap();
        match std::mem::take(state) {
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
        match std::mem::take(state) {
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
