//! Primitives for asynchronous I/O.

pub use std::io::{Error, Result, SeekFrom};

mod read;
pub use read::{Read, ReadAt, ReadAtExt, ReadAtFixed, ReadExt};

mod seek;
pub use seek::Seek;

mod write;
pub use write::{Write, WriteAt, WriteAtExt, WriteAtFixed, WriteExt};
