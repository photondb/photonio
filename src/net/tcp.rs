use std::{
    future::Future,
    io::{Error, ErrorKind, Result},
    net::{SocketAddr, ToSocketAddrs},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    time::Duration,
};

use socket2::{Domain, Socket, Type};

use crate::io::{op, Read, Write};

pub struct TcpListener(Socket);

impl TcpListener {
    fn raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }

    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        let mut last_err = None;
        for addr in addrs.to_socket_addrs()? {
            match Self::bind_addr(addr) {
                Ok(l) => return Ok(l),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| ErrorKind::InvalidInput.into()))
    }

    fn bind_addr(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        socket.set_reuse_port(true)?;
        socket.set_reuse_address(true)?;
        let sock_addr = addr.into();
        socket.bind(&sock_addr)?;
        socket.listen(1024)?;
        Ok(Self(socket))
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (fd, addr) = op::accept(self.raw_fd()).await?;
        let socket = unsafe { Socket::from_raw_fd(fd) };
        let sock_addr = addr
            .as_socket()
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid socket address"))?;
        Ok((TcpStream(socket), sock_addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        addr.as_socket()
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid socket address"))
    }
}

pub struct TcpStream(Socket);

impl TcpStream {
    fn raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }

    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        op::connect(socket.as_raw_fd(), addr.into()).await?;
        Ok(Self(socket))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        addr.as_socket()
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid socket address"))
    }

    pub fn nodelay(&self) -> Result<bool> {
        self.0.nodelay()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.0.set_nodelay(nodelay)
    }

    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        self.0.read_timeout()
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()> {
        self.0.set_read_timeout(dur)
    }

    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        self.0.write_timeout()
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> Result<()> {
        self.0.set_write_timeout(dur)
    }
}

impl Read for TcpStream {
    type ReadFuture<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        op::read(self.raw_fd(), buf)
    }
}

impl Write for TcpStream {
    type WriteFuture<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        op::write(self.raw_fd(), buf)
    }
}
