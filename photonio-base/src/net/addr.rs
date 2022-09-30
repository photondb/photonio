use std::{future::Future, io::Result, net::SocketAddr};

/// A trait for objects that can be resolved to one or more socket addresses.
///
/// This trait is an async version of [`std::net::ToSocketAddrs`].
pub trait ToSocketAddrs {
    /// An iterator over the resolved socket addresses.
    type Iter: Iterator<Item = SocketAddr>;
    /// A future that resolves to the result of [`Self::to_socket_addrs`].
    type Future: Future<Output = Result<Self::Iter>>;

    /// See [`std::net::ToSocketAddrs::to_socket_addrs`].
    fn to_socket_addrs(&self) -> Self::Future;
}
