//! A utility module to generate paths, formats, and i/o structure
//!
//! In the future this module may be versioned to allow for data
//! layout migrations.

use crate::{crypto::CipherText, dir::Dirs, utils::Id};
use std::{
    io::{self, Read},
    path::PathBuf,
};

/// Create an fs path from a db path
pub(crate) fn path(dirs: &Dirs, id: Id) -> PathBuf {
    dirs.records().join(&format!("{}.bin", id))
}

/// Serialize an entire record into a binary array
// This function should not have to exist
#[deprecated]
pub(crate) fn encode(ref txt: CipherText) -> Vec<u8> {
    bincode::serialize(txt).unwrap()
}

/// Read a big-endian encoded u32 from a stream
#[inline]
pub(crate) fn read_u32(f: &mut impl Read) -> io::Result<u32> {
    let mut len_buf: [u8; 4] = [0; 4];
    f.read_exact(&mut len_buf)?;
    Ok(u32::from_be_bytes(len_buf))
}

/// Read a (be) u32 length-prepended vector
#[inline]
pub(crate) fn read_vec(f: &mut impl Read) -> io::Result<Vec<u8>> {
    let len = read_u32(f)?;

    let mut buf = Vec::with_capacity(len as usize);
    f.read_exact(&mut buf)?;
    Ok(buf)
}

pub(crate) fn read_vec_exact(f: &mut impl Read, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(len);
    f.read_exact(&mut buf)?;
    Ok(buf)
}
