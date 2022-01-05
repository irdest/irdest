//! I/O persistence module
//!
//! This module is split into three parts.
//!
//! 1. Fundamental wire encoding types
//!
//!    These are types wrapping an encoding library to ensure fast and
//!    correct encoding and decoding to disk.  The data written is
//!    always encrypted and doesn't directly get access to the crypto
//!    engine.  In fact, because file I/O operations are synchronous,
//!    they are forked out to special worker threads that are allowed
//!    to block on operations
//!
//! 2. Chunk management system
//!
//!    Records in Alexandria are written into several chunks to avoid
//!    seeking costs.  A chunk index is kept to re-associate fragments
//!    for the same chunk.  This abstraction also de-crypts data on a
//!    need-to-know basis, having access to a user's key information
//!    to perform encryption and decryption operations.
//!
//!    This abstraction will attempt to cache as much data as is
//!    sensible (in encrypted form) to perform more efficient seek
//!    operations.  The `ChunkCache` is responsible for keeping this
//!    data up-to-date!
//!
//! 3. Record parsing system
//!
//!    On top of chunk data are encoded records.  All records are
//!    stream-encoded, meaning that they can be read on a
//!    chunk-by-chunk basis.  For example: a table record consists of
//!    rows, with each row consisting of a length prefix, followed by
//!    the associated data which can then be parsed via a fundamental
//!    wire encoding type.
//!
//!    At this stage data has already been decrypted, and thus no
//!    longer has access to the user's key information.  Data on this
//!    layer is not cached, and instead is de-crypted on the fly again
//!    if a reverse seek needs to be applied.
//!
//! When working on this module it is of utmost imperative to keep
//! these layers separated to avoid accidental data leaks.

pub(self) mod proto {
    include!(concat!(env!("OUT_DIR"), "/io/proto_gen/mod.rs"));
}

#[deprecated]
mod util;

mod cfg;
mod chunk;
mod error;
mod record;
mod wire;

pub use cfg::legacy as versions;
pub use cfg::Config;
pub(crate) use util::{Decode, Encode};
pub(self) use chunk::Chunk;
