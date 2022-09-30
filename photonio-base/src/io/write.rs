//! Primitives for asynchronous writes.

use std::{
    future::Future,
    io::{ErrorKind, Result},
};

/// Asychronous sequential writes.
pub trait Write {
    /// A future that resolves to the result of [`Self::write`].
    type Write<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    /// Writes some bytes from `buf` and returns the number of bytes written.
    fn write<'b>(&mut self, buf: &'b [u8]) -> Self::Write<'b>;
}

/// Extension methods for [`Write`].
pub trait WriteExt {
    /// A future that resolves to the result of [`Self::write_all`].
    type WriteAll<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    /// Writes all bytes from `buf`.
    fn write_all<'a: 'b, 'b>(&'a mut self, buf: &'b [u8]) -> Self::WriteAll<'b>;
}

impl<T> WriteExt for T
where
    T: Write,
{
    type WriteAll<'b> = impl Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_all<'a: 'b, 'b>(&'a mut self, mut buf: &'b [u8]) -> Self::WriteAll<'b> {
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

/// Asynchronous positional writes.
pub trait WriteAt {
    /// A future that resolves to the result of [`Self::write_at`].
    type WriteAt<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    /// Writes some bytes from `buf` at `pos` and returns the number of bytes written.
    fn write_at<'b>(&self, buf: &'b [u8], pos: u64) -> Self::WriteAt<'b>;
}

/// Extension methods for [`WriteAt`].
pub trait WriteAtExt {
    /// A future that resolves to the result of [`Self::write_all_at`].
    type WriteAllAt<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    /// Writes all bytes from `buf` at `pos`.
    fn write_all_at<'a: 'b, 'b>(&'a self, buf: &'b [u8], pos: u64) -> Self::WriteAllAt<'b>;
}

impl<T> WriteAtExt for T
where
    T: WriteAt,
{
    type WriteAllAt<'b> = impl Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_all_at<'a: 'b, 'b>(&'a self, mut buf: &'b [u8], mut pos: u64) -> Self::WriteAllAt<'b> {
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
