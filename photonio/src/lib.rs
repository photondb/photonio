//! A runtime for asynchronous filesystem and networking I/O.

#![warn(missing_docs)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

mod common;

#[cfg(any(doc, all(target_os = "linux", not(feature = "tokio"))))]
mod core;
#[cfg(any(not(target_os = "linux"), feature = "tokio"))]
mod tokio;

#[cfg(any(doc, all(target_os = "linux", not(feature = "tokio"))))]
pub use crate::core::*;
#[cfg(any(not(target_os = "linux"), feature = "tokio"))]
pub use crate::tokio::*;
