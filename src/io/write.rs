use std::{
    future::Future,
    io::{ErrorKind, Result},
};

pub trait Write {
    type Write<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    fn write<'b>(&mut self, buf: &'b [u8]) -> Self::Write<'b>;
}

pub trait WriteExt {
    type WriteExact<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_exact<'a: 'b, 'b>(&'a mut self, buf: &'b [u8]) -> Self::WriteExact<'b>;
}

impl<T> WriteExt for T
where
    T: Write,
{
    type WriteExact<'b> = impl Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_exact<'a: 'b, 'b>(&'a mut self, mut buf: &'b [u8]) -> Self::WriteExact<'b> {
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

pub trait WriteAt {
    type WriteAt<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    fn write_at<'b>(&self, buf: &'b [u8], pos: u64) -> Self::WriteAt<'b>;
}

pub trait WriteAtExt {
    type WriteExactAt<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_exact_at<'a: 'b, 'b>(&'a self, buf: &'b [u8], pos: u64) -> Self::WriteExactAt<'b>;
}

impl<T> WriteAtExt for T
where
    T: WriteAt,
{
    type WriteExactAt<'b> = impl Future<Output = Result<()>> + 'b
    where
        Self: 'b;

    fn write_exact_at<'a: 'b, 'b>(
        &'a self,
        mut buf: &'b [u8],
        mut pos: u64,
    ) -> Self::WriteExactAt<'b> {
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
