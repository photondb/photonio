//! Networking types and operations.
//!
//! This module is an async version of [`std::net`].

mod tcp;
pub use tcp::{TcpListener, TcpStream};
