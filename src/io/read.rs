use std::{future::Future, io::Result};

pub trait Read {
    type ReadFuture: Future<Output = Result<usize>>;

    fn read(&mut self, buf: &mut [u8]) -> Self::ReadFuture;
}

pub trait ReadAt {
    type ReadAtFuture: Future<Output = Result<usize>>;

    fn read_at(&self, buf: &mut [u8], pos: usize) -> Self::ReadAtFuture;
}
