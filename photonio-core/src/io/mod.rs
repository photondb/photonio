//! Primitives for asynchronous I/O.
//!
//! This module is an async version of [`std::io`].

pub use photonio_base::io::*;

mod op;
use op::{Op, OpTable};

mod driver;
pub(crate) use driver::{submit, Driver};

pub(crate) mod syscall;

use std::io::{Error, Result};

fn syscall_result(res: i32) -> Result<u32> {
    if res >= 0 {
        Ok(res as u32)
    } else {
        Err(Error::from_raw_os_error(-res))
    }
}
