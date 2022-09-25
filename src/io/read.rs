use std::{future::Future, io::Result};

pub trait Read {
    type Read<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    fn read<'b>(&mut self, buf: &'b mut [u8]) -> Self::Read<'b>;
}

pub trait ReadExt {
    type ReadExact<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

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
                let n = self.read(buf).await?;
                buf = &mut buf[n..];
            }
            Ok(())
        }
    }
}

pub trait ReadAt {
    type ReadAt<'b>: Future<Output = Result<usize>> + 'b
    where
        Self: 'b;

    fn read_at<'b>(&self, buf: &'b mut [u8], pos: u64) -> Self::ReadAt<'b>;
}

pub trait ReadAtExt {
    type ReadExactAt<'b>: Future<Output = Result<()>> + 'b
    where
        Self: 'b;

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
                let n = self.read_at(buf, pos).await?;
                buf = &mut buf[n..];
                pos += n as u64;
            }
            Ok(())
        }
    }
}
