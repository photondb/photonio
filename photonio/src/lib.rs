//! A runtime for asynchronous applications.
//!
//! There are two implementations of this runtime:
//! - The [`photonio-core`][photonio-core] crate provides an implementation based on io-uring.
//! - The [`photonio-tokio`][photonio-tokio] crate provides an implementation based on
//!   [`tokio`][tokio].
//!
//! By default, this crate uses the `photonio-core` implementation on Linux and
//! the `photonio-tokio` implementation on other platforms. To use the `photonio-tokio`
//! implementation on all platforms, enable the `tokio` feature.
//!
//! [photonio-core]: https://docs.rs/photonio-core
//! [photonio-tokio]: https://docs.rs/photonio-tokio
//! [tokio]: https://docs.rs/tokio

#![warn(missing_docs)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

#[cfg(any(doc, all(target_os = "linux", not(feature = "tokio"))))]
pub use photonio_core::*;
pub use photonio_macros::{main, test};
#[cfg(any(not(target_os = "linux"), feature = "tokio"))]
pub use photonio_tokio::*;
