//! A PhotonIO implementation based on io-uring.

#![warn(missing_docs, unreachable_pub)]
#![feature(pin_macro, io_error_more, type_alias_impl_trait)]

#[cfg(any(doc, target_os = "linux"))]
pub mod fs;
#[cfg(any(doc, target_os = "linux"))]
pub mod io;
#[cfg(any(doc, target_os = "linux"))]
pub mod net;
#[cfg(any(doc, target_os = "linux"))]
pub mod runtime;
#[cfg(any(doc, target_os = "linux"))]
pub mod task;
