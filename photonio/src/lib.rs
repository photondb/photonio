//! A runtime for asynchronous filesystem and networking I/O.

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
