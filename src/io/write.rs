use std::{future::Future, io::Result};

pub trait Write {
    type WriteFuture: Future<Output = Result<usize>>;

    fn write(&mut self, buf: &[u8]) -> Self::WriteFuture;
}

pub trait WriteAt {
    type WriteAtFuture: Future<Output = Result<usize>>;

    fn write_at(&self, buf: &[u8], pos: usize) -> Self::WriteAtFuture;
}
