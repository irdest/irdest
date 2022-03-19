use crate::{ReadCapability, BlockSize, BlockStorage, BlockKey, BlockReference, chacha20};
use thiserror::Error as ThisError;
use std::collections::VecDeque;

#[derive(ThisError, Debug)]
pub enum DecodeError {
    #[error("Invalid padding")]
    Padding,
}
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    Decode(#[from] DecodeError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Block not found in storage")]
    BlockNotFound,
}
pub type DecodeResult<T = ()> = std::result::Result<T, DecodeError>;
pub type Result<T = ()> = std::result::Result<T, Error>;

fn unpad(input: &mut Vec<u8>, block_size: BlockSize) -> DecodeResult {
    let old_len = input.len();
    loop {
        match input.pop() {
            Some(0) => (),
            Some(0x80) => return Ok(()),
            _ => return Err(DecodeError::Padding),
        }
        if old_len - input.len() > *block_size {
            return Err(DecodeError::Padding);
        }
    }
}

pub struct Decoder<'a, S: BlockStorage> {
    pub block_storage: &'a S,
}

impl<'a, S: BlockStorage> Decoder<'a, S> {
    async fn decode(&self, read_capability: &ReadCapability) -> Result<Vec<u8>> {
        let mut out = vec![];

        let mut subtrees = VecDeque::new();
        subtrees.push_back(read_capability.clone());

        while let Some(tree) = subtrees.pop_front() {
            let mut block = self.block_storage.fetch(&tree.root_reference).await?.ok_or(Error::BlockNotFound)?;
            chacha20(&mut block, &tree.root_key);

            if tree.level == 0 {
                out.append(&mut block);
            } else {
                for rk_pair_raw in block.chunks_exact(64) {
                    let has_content = rk_pair_raw.iter().any(|x| *x != 0);
                    if !has_content { break; }

                    let rk_pair = (BlockReference(rk_pair_raw[..32].try_into().unwrap()), BlockKey(rk_pair_raw[32..].try_into().unwrap()));
                    subtrees.push_back(ReadCapability::from_rk_pair(rk_pair, tree.level - 1, read_capability.block_size));
                }
            }
        }

        unpad(&mut out, read_capability.block_size)?;
        Ok(out)
    }
}

pub async fn decode<S: BlockStorage>(read_capability: &ReadCapability, block_storage: &S) -> Result<Vec<u8>> {
    let decoder = Decoder { block_storage };
    decoder.decode(read_capability).await
}
