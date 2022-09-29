//! A runtime for asynchronous filesystem and networking I/O.

#![warn(missing_docs)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;
pub mod task;

pub use photonio_macros::{main, test};
