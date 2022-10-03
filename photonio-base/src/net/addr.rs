use std::{future::Future, io::Result, net::SocketAddr};

/// Resolves to one or more socket addresses.
///
/// This trait is an async version of [`std::net::ToSocketAddrs`].
pub trait ToSocketAddrs {
    /// An iterator over the resolved [`SocketAddr`] values.
    type Iter: Iterator<Item = SocketAddr>;
    /// A future that resolves to the result of [`Self::to_socket_addrs`].
    type Future: Future<Output = Result<Self::Iter>>;

    /// Resolves this object to an iterator of [`SocketAddr`] values.
    ///
    /// See also [`std::net::ToSocketAddrs::to_socket_addrs`].
    fn to_socket_addrs(&self) -> Self::Future;
}
