// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{Block, BlockKey, BlockReference, BlockStorage, ReadCapability};
use futures_io::AsyncWrite;
use futures_util::io::AsyncWriteExt;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid padding")]
    Padding,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Block not found in storage")]
    BlockNotFound,
    #[error("Non-standard block size")]
    NonstandardBlockSize,
    #[error("Unexpected block size")]
    UnexpectedBlockSize,
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

async fn fetch_and_decrypt_block<S: BlockStorage<BS>, const BS: usize>(
    read_capability: ReadCapability,
    block_storage: &S,
) -> Result<(u8, Block<BS>)> {
    let mut block = block_storage
        .fetch(&read_capability.root_reference)
        .await?
        .ok_or(Error::BlockNotFound)?;
    block.chacha20(&read_capability.root_key);
    Ok::<_, Error>((read_capability.level, block))
}

pub async fn decode_const<S: BlockStorage<BS>, W: AsyncWrite + Unpin, const BS: usize>(
    target: &mut W,
    read_capability: &ReadCapability,
    block_storage: &S,
) -> Result<()> {
    if read_capability.block_size != BS {
        return Err(Error::UnexpectedBlockSize);
    }

    let mut subtrees = FuturesUnordered::new();
    subtrees.push(fetch_and_decrypt_block(
        read_capability.clone(),
        block_storage,
    ));

    while let Some((level, block)) = subtrees.next().await.transpose()? {
        if level == 0 {
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
                let read_capability =
                    ReadCapability::from_rk_pair(rk_pair, level - 1, read_capability.block_size);

                subtrees.push(fetch_and_decrypt_block(read_capability, block_storage));
            }
        }
    }

    Ok(())
}
