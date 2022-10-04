//! A PhotonIO implementation based on io-uring.

#![warn(missing_docs, unreachable_pub)]
#![feature(pin_macro, io_error_more, type_alias_impl_trait)]

#[cfg(target_os = "linux")]
pub mod fs;
#[cfg(target_os = "linux")]
pub mod io;
#[cfg(target_os = "linux")]
pub mod net;
#[cfg(target_os = "linux")]
pub mod runtime;
#[cfg(target_os = "linux")]
pub mod task;
