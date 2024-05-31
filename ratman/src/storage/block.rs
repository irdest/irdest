use async_eris::{Block, BlockReference, BlockSize};
use libratman::{types::Id, BlockError, RatmanError, Result};
use serde::{Deserialize, Serialize};

/// A wrapper type for storing Blocks in various parts of the code
///
/// Provide utilities to handle the const generics of the underlying
/// type, as well as storage, retrieval, and checking integrity
#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub(crate) enum StorageBlock {
    /// 1K block size
    _1K(Block<1024>),
    /// 32K block size
    _32K(Block<32768>),
}

impl StorageBlock {
    pub fn reference(&self) -> BlockReference {
        match self {
            Self::_1K(b) => b.reference(),
            Self::_32K(b) => b.reference(),
        }
    }

    /// Create a StorageBlock from a raw byte stream
    ///
    /// Optionally a sequence ID can be provided, which is used to
    /// yield more useful errors in case block lengths didn't align.
    pub fn reconstruct(block_buf: Vec<u8>) -> Result<Self> {
        match block_buf.len() {
            1024 => Ok(StorageBlock::_1K(Block::<1024>::copy_from_vec(block_buf))),
            32768 => Ok(StorageBlock::_32K(Block::<32768>::copy_from_vec(block_buf))),
            length => Err(RatmanError::Block(BlockError::InvalidLength(length))),
        }
    }

    /// Dissolve this type, yielding the block Id and underlying data
    pub fn dissolve<const L: usize>(self) -> (BlockReference, Vec<u8>) {
        let block_ref = self.reference();
        (
            block_ref,
            match self {
                Self::_1K(b) => b.to_vec(),
                Self::_32K(b) => b.to_vec(),
            },
        )
    }
}
