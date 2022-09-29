use std::{
    future::Future,
    io::{Error, ErrorKind, Result},
    net::{Shutdown, SocketAddr, ToSocketAddrs},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    time::Duration,
};

use socket2::{Domain, SockAddr, Socket, Type};

use crate::io::{syscall, Read, Write};

/// A TCP socket listening for connections.
///
/// This type is an async version of [`std::net::TcpListener`].
pub struct TcpListener(Socket);

impl TcpListener {
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

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (fd, addr) = syscall::accept(self.raw_fd()).await?;
        let socket = unsafe { Socket::from_raw_fd(fd) };
        let socket_addr = to_socket_addr(addr)?;
        Ok((TcpStream(socket), socket_addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }
}

impl TcpListener {
    fn bind_addr(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        socket.set_reuse_port(true)?;
        socket.set_reuse_address(true)?;
        let sock_addr = addr.into();
        socket.bind(&sock_addr)?;
        socket.listen(1024)?;
        Ok(Self(socket))
    }

    fn raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// A TCP stream between a local and a remote socket.
///
/// This type is an async version of [`std::net::TcpStream`].
pub struct TcpStream(Socket);

impl TcpStream {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        syscall::connect(socket.as_raw_fd(), addr.into()).await?;
        Ok(Self(socket))
    }

    pub async fn shutdown(&self, how: Shutdown) -> Result<()> {
        let flags = match how {
            Shutdown::Both => libc::SHUT_RDWR,
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
        };
        syscall::shutdown(self.raw_fd(), flags).await.map(|_| ())
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.peer_addr()?;
        to_socket_addr(addr)
    }

    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
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

impl TcpStream {
    fn raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Read for TcpStream {
    type Read<'b> = impl Future<Output = Result<usize>> + 'b;

    fn read<'b>(&mut self, buf: &'b mut [u8]) -> Self::Read<'b> {
        syscall::read(self.raw_fd(), buf)
    }
}

impl Write for TcpStream {
    type Write<'b> = impl Future<Output = Result<usize>> + 'b;

    fn write<'b>(&mut self, buf: &'b [u8]) -> Self::Write<'b> {
        syscall::write(self.raw_fd(), buf)
    }
}

fn to_socket_addr(addr: SockAddr) -> Result<SocketAddr> {
    addr.as_socket()
        .ok_or_else(|| Error::new(ErrorKind::Other, "invalid socket address"))
}
