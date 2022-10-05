//! Primitives for asynchronous seeks.

use std::{
    future::Future,
    io::{Result, SeekFrom},
};

/// Seeks to a position in an object.
pub trait Seek {
    /// A future that resolves to the result of [`Self::seek`].
    type Seek: Future<Output = Result<u64>>;

    /// Seeks to a given position in this object.
    ///
    /// Returns the new position from the start of this object.
    fn seek(&mut self, pos: SeekFrom) -> Self::Seek;
}
