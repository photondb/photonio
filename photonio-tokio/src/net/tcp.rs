use std::{future::Future, io::Result, net::SocketAddr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net,
};

use super::ToSocketAddrs;
use crate::io::{Read, Write};

#[derive(Debug)]
pub struct TcpListener(net::TcpListener);

impl TcpListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let addrs: Vec<_> = addr.to_socket_addrs().await?.collect();
        Ok(Self(net::TcpListener::bind(addrs.as_slice()).await?))
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (stream, addr) = self.0.accept().await?;
        Ok((TcpStream(stream), addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn ttl(&self) -> Result<u32> {
        self.0.ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.0.set_ttl(ttl)
    }
}

#[derive(Debug)]
pub struct TcpStream(net::TcpStream);

impl TcpStream {
    pub async fn connect(addr: impl net::ToSocketAddrs) -> Result<Self> {
        Ok(Self(net::TcpStream::connect(addr).await?))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.0.peer_addr()
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
}

impl Read for TcpStream {
    type Read<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a> {
        self.0.read(buf)
    }
}

impl Write for TcpStream {
    type Write<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a> {
        self.0.write(buf)
    }
}
