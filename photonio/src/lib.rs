//! An efficient runtime for asynchronous applications.
//!
//! There are two implementations of this runtime:
//! - The [`photonio-uring`][photonio-uring] crate provides an implementation for Linux based on
//!   [`io_uring`][io_uring].
//! - The [`photonio-tokio`][photonio-tokio] crate provides an implementation for other platforms
//!   based on [`tokio`][tokio].
//!
//! By default, this crate uses the `photonio-uring` implementation on Linux and
//! the `photonio-tokio` implementation on other platforms. To use the `photonio-tokio`
//! implementation on all platforms, enable the `tokio` feature.
//!
//! [photonio-uring]: https://docs.rs/photonio-uring
//! [photonio-tokio]: https://docs.rs/photonio-tokio
//! [io_uring]: https://unixism.net/loti/
//! [tokio]: https://docs.rs/tokio
//!
//! ## Limitations
//!
//! - Dropping an unfinished future for asynchronous filesystem or networking operations will result
//!   in a panic. However, this behavior might be change in the future.
//! - The current multi-thread runtime uses a naive round-robin fashion to schedule tasks. A
//!   work-stealing scheduler will be added in the future.

#![warn(missing_docs, unreachable_pub)]
#![feature(pin_macro, io_error_more, type_alias_impl_trait)]

pub use photonio_macros::{main, test};
#[cfg(any(feature = "tokio", not(target_os = "linux")))]
pub use photonio_tokio::*;
#[cfg(all(not(feature = "tokio"), target_os = "linux"))]
pub use photonio_uring::*;
