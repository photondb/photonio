use std::{io::Result, net::SocketAddr};

use tokio::net;

use super::ToSocketAddrs;

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
