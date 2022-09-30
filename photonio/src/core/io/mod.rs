//! Primitives for asynchronous I/O.
//!
//! This module is an async version of [`std::io`].

pub use crate::common::io::*;

mod driver;
pub(crate) use driver::{submit, Driver};

pub(crate) mod syscall;
