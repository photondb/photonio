//! A runtime for asynchronous filesystem and networking I/O.
//!
//! There are two implementations of this runtime:
//! - The [`photonio-core`][photonio-core] crate provide an implementation based on io-uring.
//! - The [`photonio-tokio`][photonio-tokio] crate provide an implementation based on
//!   [`tokio`][tokio].
//!
//! By default, this crate use the `photonio-core` implementation on Linux and use
//! the `photonio-tokio` implementation on other platforms.
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
