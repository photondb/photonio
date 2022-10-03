//! Primitives for asynchronous reads.

use std::{
    future::Future,
    io::{ErrorKind, Result},
};

/// Reads some bytes from an object.
pub trait Read {
    /// A future that resolves to the result of [`Self::read`].
    type Read<'a>: Future<Output = Result<usize>> + 'a
    where
        Self: 'a;

    /// Reads some bytes from this object into `buf`.
    ///
    /// Returns the number of bytes read.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a>;
}

/// Provides extension methods for [`Read`].
pub trait ReadExt {
    /// A future that resolves to the result of [`Self::read_exact`].
    type ReadExact<'a>: Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    /// Reads the exact number of bytes from this object to fill `buf`.
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadExact<'a>;
}

impl<T> ReadExt for T
where
    T: Read,
{
    type ReadExact<'a> = impl Future<Output = Result<()>> + 'a where Self: 'a;

    fn read_exact<'a>(&'a mut self, mut buf: &'a mut [u8]) -> Self::ReadExact<'a> {
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

/// Reads some bytes from an object at a given position.
pub trait ReadAt {
    /// A future that resolves to the result of [`Self::read_at`].
    type ReadAt<'a>: Future<Output = Result<usize>> + 'a
    where
        Self: 'a;

    /// Reads some bytes from this object at `pos` into `buf`.
    ///
    /// Returns the number of bytes read.
    fn read_at<'a>(&'a self, buf: &'a mut [u8], pos: u64) -> Self::ReadAt<'a>;
}

/// Provides extension methods for [`ReadAt`].
pub trait ReadAtExt {
    /// A future that resolves to the result of [`Self::read_exact_at`].
    type ReadExactAt<'a>: Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    /// Reads the exact number of bytes from this object at `pos` to fill `buf`.
    fn read_exact_at<'a>(&'a self, buf: &'a mut [u8], pos: u64) -> Self::ReadExactAt<'a>;
}

impl<T> ReadAtExt for T
where
    T: ReadAt,
{
    type ReadExactAt<'a> = impl Future<Output = Result<()>> + 'a where Self: 'a;

    fn read_exact_at<'a>(&'a self, mut buf: &'a mut [u8], mut pos: u64) -> Self::ReadExactAt<'a> {
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
