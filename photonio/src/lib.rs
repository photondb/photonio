//! A runtime for asynchronous applications.
//!
//! There are two implementations of this runtime:
//! - The [`photonio-uring`][photonio-uring] crate provides an implementation based on io-uring.
//! - The [`photonio-tokio`][photonio-tokio] crate provides an implementation based on
//!   [`tokio`][tokio].
//!
//! By default, this crate uses the `photonio-uring` implementation on Linux and
//! the `photonio-tokio` implementation on other platforms. To use the `photonio-tokio`
//! implementation on all platforms, enable the `tokio` feature.
//!
//! [photonio-uring]: https://docs.rs/photonio-uring
//! [photonio-tokio]: https://docs.rs/photonio-tokio
//! [tokio]: https://docs.rs/tokio

#![deny(unused_must_use)]
#![warn(missing_docs, unreachable_pub)]
#![allow(clippy::new_without_default)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

pub use photonio_macros::{main, test};
#[cfg(any(not(target_os = "linux"), feature = "tokio"))]
pub use photonio_tokio::*;
#[cfg(any(doc, all(target_os = "linux", not(feature = "tokio"))))]
pub use photonio_uring::*;
