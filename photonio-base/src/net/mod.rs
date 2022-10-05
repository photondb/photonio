//! Primitives for asynchronous networking operations.

pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6};

mod addr;
pub use addr::ToSocketAddrs;
