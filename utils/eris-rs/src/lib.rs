// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod dec;
mod enc;
mod serde_util;

pub use dec::{decode, decode_const, Error, Result};
pub use enc::{encode, encode_const, BlockSize, Encoder};

#[cfg(test)]
mod tests;

/// The target specification this implementation supports
///
/// Blocks encoded with a later specification MAY still correctly
/// decode, but no guarantees can be made.  If you find that some
/// content encoded with a compatible library does NOT correctly
/// decode, please get in touch!
pub const TARGET_SPECIFICATION: &str = "1.0.0";

use async_trait::async_trait;
use blake2::{digest::consts::U32, Blake2b, Digest};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use derive_more::{DebugCustom, Deref, DerefMut, Display, From};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn display_base32(bytes: &[u8]) -> String {
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, bytes)
}

// Really only useful for decoding variable length test data
#[cfg(test)]
fn vardecode_base32(s: &str) -> Result<Vec<u8>> {
    let buf = base32::decode(base32::Alphabet::RFC4648 { padding: false }, s)
        .ok_or(Error::InvalidBase32)?;
    Ok(buf)
}

fn decode_base32<const BS: usize>(s: &str) -> Result<[u8; BS]> {
    match base32::decode(base32::Alphabet::RFC4648 { padding: false }, s)
        .ok_or(Error::InvalidBase32)
    {
        Ok(v) if v.len() == BS => Ok(v.try_into().map_err(|_| Error::InvalidBase32)?),
        Ok(_) => Err(Error::UnexpectedBlockSize),
        Err(e) => Err(e),
    }
}

/// A 32 byte block reference identifier
#[derive(Clone, PartialEq, Eq, Hash, Deref, From, Display, DebugCustom, Serialize, Deserialize)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct BlockReference([u8; 32]);

impl BlockReference {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(
            bytes
                .try_into()
                .expect("not enough bytes for a block reference"),
        )
    }
}

impl TryFrom<&String> for BlockReference {
    type Error = Error;
    fn try_from(s: &String) -> Result<BlockReference> {
        let buf = decode_base32(s.as_str())?;
        Ok(BlockReference(buf))
    }
}

/// Represents a 32 byte ChaCha20 key for encryption
#[derive(Clone, Deref, From, Display, DebugCustom)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct BlockKey([u8; 32]);

impl BlockKey {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&String> for BlockKey {
    type Error = Error;
    fn try_from(s: &String) -> Result<BlockKey> {
        let buf = decode_base32(s.as_str())?;
        Ok(BlockKey(buf))
    }
}

#[doc(hidden)] // Only exposed for tests
pub type RKPair = (BlockReference, BlockKey);

/// An encoded data block of size `BLOCKSIZE`
#[derive(Clone, Deref, DerefMut, From, Display, DebugCustom, Serialize, Deserialize)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct Block<const BS: usize>(#[serde(with = "serde_util")] [u8; BS]);

impl<const BS: usize> Block<BS> {
    pub fn as_slice(&self) -> &[u8; BS] {
        &self.0
    }

    /// Take a `Vec<u8>` and move its contents into an array
    pub fn copy_from_vec(vec: Vec<u8>) -> Self {
        let mut buf = [0; BS];
        buf.copy_from_slice(vec.as_slice());
        Self(buf)
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl<const BS: usize> TryFrom<&String> for Block<BS> {
    type Error = Error;
    fn try_from(s: &String) -> Result<Block<BS>> {
        let buf = decode_base32(s.as_str())?;
        Ok(Block(buf))
    }
}

#[async_trait]
pub trait BlockStorage<const BS: usize> {
    async fn store(&mut self, block: &Block<BS>) -> std::io::Result<()>;
    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Block<BS>>>;
}

pub type MemoryStorage = HashMap<BlockReference, Vec<u8>>;

#[async_trait]
impl<const BS: usize> BlockStorage<BS> for MemoryStorage {
    async fn store(&mut self, block: &Block<BS>) -> std::io::Result<()> {
        self.insert(block.reference(), block.0.to_vec());
        Ok(())
    }

    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Block<BS>>> {
        self.get(reference)
            .map(|x| -> std::io::Result<_> {
                let arr: [u8; BS] = x.clone().try_into().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "Block has unexpected size")
                })?;
                Ok(arr.into())
            })
            .transpose()
    }
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

// replace with usize::log2 once its stable
fn log_2(x: usize) -> u32 {
    assert!(x > 0);
    num_bits::<usize>() as u32 - x.leading_zeros() - 1
}

#[derive(Clone, Debug)]
pub struct ReadCapability {
    pub root_reference: BlockReference,
    pub root_key: BlockKey,
    pub level: u8,
    pub block_size: usize,
}

impl ReadCapability {
    pub(crate) fn from_rk_pair(rk_pair: RKPair, level: u8, block_size: usize) -> ReadCapability {
        ReadCapability {
            root_reference: rk_pair.0,
            root_key: rk_pair.1,
            level,
            block_size,
        }
    }

    pub fn binary(&self) -> Vec<u8> {
        let mut out = vec![];
        out.push(log_2(self.block_size) as u8);
        out.push(self.level);
        out.extend_from_slice(&*self.root_reference);
        out.extend_from_slice(&*self.root_key);
        out
    }

    pub fn from_binary(buf: &[u8]) -> Option<ReadCapability> {
        if buf.len() != 1 + 1 + 32 + 32 {
            return None;
        }
        let block_size = 2usize.pow(buf[0].into());
        let level = buf[1];
        let root_reference_bytes: [u8; 32] = buf[2..34].try_into().unwrap();
        let root_key_bytes: [u8; 32] = buf[34..66].try_into().unwrap();
        let root_reference = BlockReference::from(root_reference_bytes);
        let root_key = BlockKey::from(root_key_bytes);
        Some(Self {
            block_size,
            level,
            root_reference,
            root_key,
        })
    }

    pub fn urn(&self) -> String {
        format!("urn:erisx2:{}", &display_base32(&self.binary()))
    }
}

impl<const BS: usize> Block<BS> {
    pub fn reference(&self) -> BlockReference {
        let mut hasher = Blake2b::<U32>::new();
        Digest::update(&mut hasher, &**self);
        BlockReference(hasher.finalize().into())
    }

    pub(crate) fn chacha20(&mut self, key: &BlockKey) {
        let nonce = [0; 12];
        let mut cipher = ChaCha20::new(&(**key).into(), &nonce.into());
        cipher.apply_keystream(&mut **self);
    }
}
