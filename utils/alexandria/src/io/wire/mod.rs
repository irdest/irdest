//! Wire encoding module
//!
//! This module wraps around an encoding library (currently protobuf)
//! to read and write data to disk.

mod encrypted;
pub use encrypted::*;

mod chunk;
pub use chunk::*;

mod table;
pub use table::*;
