#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;

pub use photonio_macros::{main, test};
