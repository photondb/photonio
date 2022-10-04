//! A PhotonIO implementation based on io-uring.

#![warn(missing_docs, unreachable_pub)]
#![feature(
    pin_macro,
    io_error_more,
    type_alias_impl_trait,
    generic_associated_types
)]

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;
pub mod task;
