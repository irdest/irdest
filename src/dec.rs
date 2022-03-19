use crate::{ReadCapability, BlockStorage, BlockKey, BlockReference, chacha20};
use thiserror::Error as ThisError;
use std::collections::VecDeque;
use futures_lite::io::{AsyncWrite, AsyncWriteExt};

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid padding")]
    Padding,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Block not found in storage")]
    BlockNotFound,
}

pub type Result<T = ()> = std::result::Result<T, Error>;

fn unpad(input: &mut Vec<u8>, block_size: usize) -> Result {
    let old_len = input.len();
    loop {
        match input.pop() {
            Some(0) => (),
            Some(0x80) => return Ok(()),
            _ => return Err(Error::Padding),
        }
        if old_len - input.len() > block_size {
            return Err(Error::Padding);
        }
    }
}

pub async fn decode<S: BlockStorage, W: AsyncWrite + Unpin>(target: &mut W, read_capability: &ReadCapability, block_storage: &S) -> Result<()> {
    let mut subtrees = VecDeque::new();
    subtrees.push_back(read_capability.clone());

    while let Some(tree) = subtrees.pop_front() {
        let mut block = block_storage.fetch(&tree.root_reference).await?.ok_or(Error::BlockNotFound)?;
        chacha20(&mut block, &tree.root_key);

        if tree.level == 0 {
            if subtrees.len() == 0 {
                // this is the last block
                unpad(&mut block, read_capability.block_size)?;
            }
            target.write_all(&block).await?;
        } else {
            for rk_pair_raw in block.chunks_exact(64) {
                let has_content = rk_pair_raw.iter().any(|x| *x != 0);
                if !has_content { break; }

                let rk_pair = (BlockReference(rk_pair_raw[..32].try_into().unwrap()), BlockKey(rk_pair_raw[32..].try_into().unwrap()));
                subtrees.push_back(ReadCapability::from_rk_pair(rk_pair, tree.level - 1, read_capability.block_size));
            }
        }
    }

    Ok(())
}
