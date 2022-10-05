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

// FIXME: Support async DNS resolution.
impl<T: std::net::ToSocketAddrs> ToSocketAddrs for T {
    type Iter = <T as std::net::ToSocketAddrs>::Iter;
    type Future = std::future::Ready<Result<Self::Iter>>;

    fn to_socket_addrs(&self) -> Self::Future {
        std::future::ready(self.to_socket_addrs())
    }
}
