// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{BlockKey, BlockReference, BlockStorage, ReadCapability};
use futures_lite::io::{AsyncWrite, AsyncWriteExt};
use std::collections::VecDeque;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("invalid padding")]
    Padding,
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("a block was not found in storage")]
    BlockNotFound,
    #[error("non-standard block size")]
    NonstandardBlockSize,
    #[error("unexpected block size")]
    UnexpectedBlockSize,
    /// Mostly used for testing
    #[error("invalid base32 encoding")]
    InvalidBase32,
}

pub type Result<T = ()> = std::result::Result<T, Error>;

fn unpad(input: &mut &[u8]) -> Result {
    loop {
        if input.len() == 0 {
            return Err(Error::Padding);
        }
        let next = input[input.len() - 1];
        *input = &mut &input[..input.len() - 1];
        match next {
            0 => (),
            0x80 => return Ok(()),
            _ => return Err(Error::Padding),
        }
    }
}

/// Decode a message from its `ReadCapability`, a block storage
/// medium, and an async write medium.
pub async fn decode<S: BlockStorage<1024> + BlockStorage<{ 32 * 1024 }>, W: AsyncWrite + Unpin>(
    target: &mut W,
    read_capability: &ReadCapability,
    block_storage: &S,
) -> Result<()> {
    match read_capability.block_size {
        1024 => decode_const::<_, _, 1024>(target, read_capability, block_storage).await,
        32768 => decode_const::<_, _, 32768>(target, read_capability, block_storage).await,
        _ => Err(Error::NonstandardBlockSize),
    }
}

pub async fn decode_const<S: BlockStorage<BS>, W: AsyncWrite + Unpin, const BS: usize>(
    target: &mut W,
    read_capability: &ReadCapability,
    block_storage: &S,
) -> Result<()> {
    if read_capability.block_size != BS {
        return Err(Error::UnexpectedBlockSize);
    }

    let mut subtrees = VecDeque::new();
    subtrees.push_back(read_capability.clone());

    while let Some(tree) = subtrees.pop_front() {
        let mut block = block_storage
            .fetch(&tree.root_reference)
            .await?
            .ok_or(Error::BlockNotFound)?;
        block.chacha20(&tree.root_key);

        if tree.level == 0 {
            let mut block = (*block).as_slice();
            if subtrees.len() == 0 {
                // this is the last block, unpad
                unpad(&mut block)?;
            }
            target.write_all(block).await?;
        } else {
            for rk_pair_raw in block.chunks_exact(64) {
                let has_content = rk_pair_raw.iter().any(|x| *x != 0);
                if !has_content {
                    break;
                }

                let rk_pair = (
                    BlockReference(rk_pair_raw[..32].try_into().unwrap()),
                    BlockKey(rk_pair_raw[32..].try_into().unwrap()),
                );
                subtrees.push_back(ReadCapability::from_rk_pair(
                    rk_pair,
                    tree.level - 1,
                    read_capability.block_size,
                ));
            }
        }
    }

    Ok(())
}
