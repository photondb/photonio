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
#[derive(Debug)]
pub struct TcpListener(Socket);

impl TcpListener {
    /// See also [`std::net::TcpListener::bind`].
    pub async fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        let mut last_err = None;
        for addr in addrs.to_socket_addrs()? {
            match listen_addr(addr) {
                Ok(l) => return Ok(Self(l)),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| ErrorKind::InvalidInput.into()))
    }

    /// See also [`std::net::TcpListener::accept`].
    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (fd, addr) = syscall::accept(self.fd()).await?;
        let socket = unsafe { Socket::from_raw_fd(fd) };
        let socket_addr = to_socket_addr(addr)?;
        Ok((TcpStream(socket), socket_addr))
    }

    /// See also [`std::net::TcpListener::local_addr`].
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    /// See also [`std::net::TcpListener::ttl`].
    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    /// See also [`std::net::TcpListener::set_ttl`].
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }
}

impl TcpListener {
    fn fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// A TCP stream between a local and a remote socket.
///
/// This type is an async version of [`std::net::TcpStream`].
#[derive(Debug)]
pub struct TcpStream(Socket);

impl TcpStream {
    /// See also [`std::net::TcpStream::connect`].
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        syscall::connect(socket.as_raw_fd(), addr.into()).await?;
        Ok(Self(socket))
    }

    /// See also [`std::net::TcpStream::shutdown`].
    pub async fn shutdown(&self, how: Shutdown) -> Result<()> {
        let flags = match how {
            Shutdown::Both => libc::SHUT_RDWR,
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
        };
        syscall::shutdown(self.fd(), flags).await.map(|_| ())
    }

    /// See also [`std::net::TcpStream::local_addr`].
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    /// See also [`std::net::TcpStream::peer_addr`].
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.peer_addr()?;
        to_socket_addr(addr)
    }

    /// See also [`std::net::TcpStream::ttl`].
    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    /// See also [`std::net::TcpStream::set_ttl`].
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }

    /// See also [`std::net::TcpStream::nodelay`].
    pub fn nodelay(&self) -> Result<bool> {
        self.0.nodelay()
    }

    /// See also [`std::net::TcpStream::set_nodelay`].
    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.0.set_nodelay(nodelay)
    }

    /// See also [`std::net::TcpStream::read_timeout`].
    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// See also [`std::net::TcpStream::set_read_timeout`].
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// See also [`std::net::TcpStream::write_timeout`].
    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// See also [`std::net::TcpStream::set_write_timeout`].
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> Result<()> {
        self.0.set_write_timeout(dur)
    }
}

impl TcpStream {
    fn fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Read for TcpStream {
    type Read<'b> = impl Future<Output = Result<usize>> + 'b;

    fn read<'b>(&mut self, buf: &'b mut [u8]) -> Self::Read<'b> {
        syscall::read(self.fd(), buf)
    }
}

impl Write for TcpStream {
    type Write<'b> = impl Future<Output = Result<usize>> + 'b;

    fn write<'b>(&mut self, buf: &'b [u8]) -> Self::Write<'b> {
        syscall::write(self.fd(), buf)
    }
}

fn listen_addr(addr: SocketAddr) -> Result<Socket> {
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
    socket.set_reuse_port(true)?;
    socket.set_reuse_address(true)?;
    let sock_addr = addr.into();
    socket.bind(&sock_addr)?;
    socket.listen(1024)?;
    Ok(socket)
}

fn to_socket_addr(addr: SockAddr) -> Result<SocketAddr> {
    addr.as_socket()
        .ok_or_else(|| Error::new(ErrorKind::Other, "invalid socket address"))
}
