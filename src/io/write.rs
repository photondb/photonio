use std::{future::Future, io::Result};

pub trait Write {
    type WriteFuture<'a>: Future<Output = Result<usize>> + 'a
    where
        Self: 'a;

    fn write<'a>(&mut self, buf: &'a [u8]) -> Self::WriteFuture<'a>;
}

pub trait WriteAt {
    type WriteAtFuture: Future<Output = Result<usize>>;

    fn write_at(&self, buf: &[u8], pos: usize) -> Self::WriteAtFuture;
}
