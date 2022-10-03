use std::{
    future::Future,
    io::{Error, ErrorKind, Result},
    net::{Shutdown, SocketAddr, ToSocketAddrs},
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd},
};

use socket2::{Domain, SockAddr, Socket, Type};

use crate::{
    io::{Read, Write},
    runtime::syscall,
};

/// A TCP socket listening for connections.
///
/// This type is an async version of [`std::net::TcpListener`].
#[derive(Debug)]
pub struct TcpListener(Socket);

impl TcpListener {
    /// Creates a TCP socket bound to the specified address.
    ///
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

    /// Accepts a new connection from this listener.
    ///
    /// See also [`std::net::TcpListener::accept`].
    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (fd, addr) = syscall::accept(self.fd()).await?;
        let stream = unsafe { TcpStream::from_raw_fd(fd.into_raw_fd()) };
        let socket_addr = to_socket_addr(addr)?;
        Ok((stream, socket_addr))
    }

    /// Returns the local socket address.
    ///
    /// See also [`std::net::TcpListener::local_addr`].
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    /// Gets the value of the `IP_TTL` option on this socket.
    ///
    /// See also [`std::net::TcpListener::ttl`].
    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    /// Sets the value of the `IP_TTL` option on this socket.
    ///
    /// See also [`std::net::TcpListener::set_ttl`].
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }
}

impl TcpListener {
    fn fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }
}

impl AsFd for TcpListener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.0.as_raw_fd()) }
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(Socket::from_raw_fd(fd))
    }
}

impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

/// A TCP stream between a local and a remote socket.
///
/// This type is an async version of [`std::net::TcpStream`].
#[derive(Debug)]
pub struct TcpStream(Socket);

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// See also [`std::net::TcpStream::connect`].
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        let stream = Self(socket);
        syscall::connect(stream.fd(), addr.into()).await?;
        Ok(stream)
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// See also [`std::net::TcpStream::shutdown`].
    pub async fn shutdown(&self, how: Shutdown) -> Result<()> {
        let flags = match how {
            Shutdown::Both => libc::SHUT_RDWR,
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
        };
        syscall::shutdown(self.fd(), flags).await.map(|_| ())
    }

    /// Returns the socket address of the local half of this connection.
    ///
    /// See also [`std::net::TcpStream::local_addr`].
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.local_addr()?;
        to_socket_addr(addr)
    }

    /// Returns the socket address of the remote peer of this connection.
    ///
    /// See also [`std::net::TcpStream::peer_addr`].
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let addr = self.0.peer_addr()?;
        to_socket_addr(addr)
    }

    /// Gets the value of the `IP_TTL` option on this socket.
    ///
    /// See also [`std::net::TcpStream::ttl`].
    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    /// Sets the value of the `IP_TTL` option on this socket.
    ///
    /// See also [`std::net::TcpStream::set_ttl`].
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// See also [`std::net::TcpStream::nodelay`].
    pub fn nodelay(&self) -> Result<bool> {
        self.0.nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// See also [`std::net::TcpStream::set_nodelay`].
    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.0.set_nodelay(nodelay)
    }
}

impl TcpStream {
    fn fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }
}

impl AsFd for TcpStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.0.as_raw_fd()) }
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(Socket::from_raw_fd(fd))
    }
}

impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Read for TcpStream {
    type Read<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a> {
        syscall::read(self.fd(), buf)
    }
}

impl Write for TcpStream {
    type Write<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a> {
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
