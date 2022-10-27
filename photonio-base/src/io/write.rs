//! Primitives for asynchronous writes.

use std::{
    future::Future,
    io::{ErrorKind, Result},
};

/// Writes some bytes into an object.
pub trait Write {
    /// A future that resolves to the result of [`Self::write`].
    type Write<'a>: Future<Output = Result<usize>> + 'a + Send
    where
        Self: 'a;

    /// Writes some bytes from `buf` into this object.
    ///
    /// Returns the number of bytes written.
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a>;
}

/// Provides extension methods for [`Write`].
pub trait WriteExt {
    /// A future that resolves to the result of [`Self::write_all`].
    type WriteAll<'a>: Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    /// Writes all bytes from `buf` into this object.
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteAll<'a>;
}

impl<T> WriteExt for T
where
    T: Write,
{
    type WriteAll<'a> = impl Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    fn write_all<'a>(&'a mut self, mut buf: &'a [u8]) -> Self::WriteAll<'a> {
        async move {
            while !buf.is_empty() {
                match self.write(buf).await {
                    Ok(0) => return Err(ErrorKind::WriteZero.into()),
                    Ok(n) => buf = &buf[n..],
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }
}

/// Writes some bytes into an object at a given position.
pub trait WriteAt {
    /// A future that resolves to the result of [`Self::write_at`].
    type WriteAt<'a>: Future<Output = Result<usize>> + 'a
    where
        Self: 'a;

    /// Writes some bytes from `buf` into this object at `pos`.
    ///
    /// Returns the number of bytes written.
    fn write_at<'a>(&'a self, buf: &'a [u8], pos: u64) -> Self::WriteAt<'a>;
}

/// Provides extension methods for [`WriteAt`].
pub trait WriteAtExt {
    /// A future that resolves to the result of [`Self::write_all_at`].
    type WriteAllAt<'a>: Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    /// Writes all bytes from `buf` into this object at `pos`.
    fn write_all_at<'a>(&'a self, buf: &'a [u8], pos: u64) -> Self::WriteAllAt<'a>;
}

impl<T> WriteAtExt for T
where
    T: WriteAt,
{
    type WriteAllAt<'a> = impl Future<Output = Result<()>> + 'a
    where
        Self: 'a;

    fn write_all_at<'a>(&'a self, mut buf: &'a [u8], mut pos: u64) -> Self::WriteAllAt<'a> {
        async move {
            while !buf.is_empty() {
                match self.write_at(buf, pos).await {
                    Ok(0) => return Err(ErrorKind::WriteZero.into()),
                    Ok(n) => {
                        buf = &buf[n..];
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
