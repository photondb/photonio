//! Types for asynchronous reads.

use std::{
    future::Future,
    io::{ErrorKind, Result},
};

/// Asynchronous sequential reads.
pub trait Read {
    /// A future that resolves to the result of [`Self::read`].
    type Read<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    /// Reads some bytes into `buf` and returns the number of bytes read.
    fn read<'b>(&mut self, buf: &'b mut [u8]) -> Self::Read<'b>;
}

/// Extension methods for [`Read`].
pub trait ReadExt {
    /// A future that resolves to the result of [`Self::read_exact`].
    type ReadExact<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    /// Reads the exact number of bytes required to fill `buf`.
    fn read_exact<'a: 'b, 'b>(&'a mut self, buf: &'b mut [u8]) -> Self::ReadExact<'b>;
}

impl<T> ReadExt for T
where
    T: Read,
{
    type ReadExact<'b> = impl Future<Output = Result<()>> + 'b where Self: 'b;

    fn read_exact<'a: 'b, 'b>(&'a mut self, mut buf: &'b mut [u8]) -> Self::ReadExact<'b> {
        async move {
            while !buf.is_empty() {
                match self.read(buf).await {
                    Ok(0) => return Err(ErrorKind::UnexpectedEof.into()),
                    Ok(n) => buf = &mut buf[n..],
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }
}

/// Asynchronous positional reads.
pub trait ReadAt {
    /// A future that resolves to the result of [`Self::read_at`].
    type ReadAt<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    /// Reads some bytes into `buf` at `pos` and returns the number of bytes read.
    fn read_at<'b>(&self, buf: &'b mut [u8], pos: u64) -> Self::ReadAt<'b>;
}

/// Extension methods for [`ReadAt`].
pub trait ReadAtExt {
    /// A future that resolves to the result of [`Self::read_exact_at`].
    type ReadExactAt<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    /// Reads the exact number of bytes required to fill `buf` at `pos`.
    fn read_exact_at<'a: 'b, 'b>(&'a self, buf: &'b mut [u8], pos: u64) -> Self::ReadExactAt<'b>;
}

impl<T> ReadAtExt for T
where
    T: ReadAt,
{
    type ReadExactAt<'b> = impl Future<Output = Result<()>> + 'b where Self: 'b;

    fn read_exact_at<'a: 'b, 'b>(
        &'a self,
        mut buf: &'b mut [u8],
        mut pos: u64,
    ) -> Self::ReadExactAt<'b> {
        async move {
            while !buf.is_empty() {
                match self.read_at(buf, pos).await {
                    Ok(0) => return Err(ErrorKind::UnexpectedEof.into()),
                    Ok(n) => {
                        buf = &mut buf[n..];
                        pos += n as u64;
                    }
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }
}
