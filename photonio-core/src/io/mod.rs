//! Primitives for asynchronous I/O.
//!
//! This module is an async version of [`std::io`].

pub use photonio_base::io::*;

mod op;
use op::{Op, OpTable};

mod driver;
pub(crate) use driver::{submit, Driver};

pub(crate) mod syscall;