//! A utility module to generate paths, formats, and i/o structure
//!
//! In the future this module may be versioned to allow for data
//! layout migrations.

use crate::{crypto::CipherText, dir::Dirs, utils::Id};
use std::path::PathBuf;

/// Create an fs path from a db path
pub(crate) fn path(dirs: &Dirs, id: Id) -> PathBuf {
    dirs.records().join(&format!("{}.bin", id))
}

/// Serialize an entire record into a binary array
pub(crate) fn encode(ref txt: CipherText) -> Vec<u8> {
    bincode::serialize(txt).unwrap()
}
