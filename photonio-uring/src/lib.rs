//! A PhotonIO implementation based on io-uring.

#![deny(unused_must_use)]
#![warn(missing_docs, unreachable_pub)]
#![allow(clippy::new_without_default)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;
pub mod task;
