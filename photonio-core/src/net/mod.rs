//! Primitives for asynchronous networking operations.
//!
//! This module is an async version of [`std::net`].

mod tcp;
pub use tcp::{TcpListener, TcpStream};
