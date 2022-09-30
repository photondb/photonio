//! Primitives for asynchronous I/O.
//!
//! This module is an async version of [`std::io`].

mod read;
pub use read::{Read, ReadAt, ReadAtExt, ReadExt};

mod write;
pub use write::{Write, WriteAt, WriteAtExt, WriteExt};

mod driver;
pub(crate) use driver::{submit, Driver};

pub(crate) mod syscall;
