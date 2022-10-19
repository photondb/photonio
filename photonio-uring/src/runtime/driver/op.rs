use std::{
    future::Future,
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

use super::OpTable;

pub(crate) struct Op {
    table: OpTable,
    index: usize,
    is_finished: bool,
}

impl Op {
    pub(super) fn new(table: OpTable, index: usize) -> Self {
        Self {
            table,
            index,
            is_finished: false,
        }
    }
}

impl Drop for Op {
    fn drop(&mut self) {
        assert!(self.is_finished);
    }
}

impl Future for Op {
    type Output = Result<u32>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let index = self.index;
        self.table.poll(index, cx.waker()).map(|v| {
            self.is_finished = true;
            v
        })
    }
}
