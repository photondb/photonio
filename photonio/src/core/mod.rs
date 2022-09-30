//! The core of the PhotonIO runtime.

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;
pub mod task;

pub use photonio_macros::{main, test};
