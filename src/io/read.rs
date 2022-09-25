use std::{future::Future, io::Result};

pub trait Read {
    type ReadFuture<'a>: Future<Output = Result<usize>> + 'a
    where
        Self: 'a;

    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;
}

pub trait ReadAt {
    type ReadAtFuture: Future<Output = Result<usize>>;

    fn read_at(&self, buf: &mut [u8], pos: usize) -> Self::ReadAtFuture;
}
