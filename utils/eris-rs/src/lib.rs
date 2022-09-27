mod enc;
pub use enc::{Encoder, encode, encode_const, BlockSize};
mod dec;
pub use dec::{decode, decode_const, Error, Result};

use blake2::{Blake2b, Digest, digest::consts::U32};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use async_trait::async_trait;
use derive_more::{Deref, DerefMut, From, Display, DebugCustom};
use std::collections::HashMap;

fn display_base32(bytes: &[u8]) -> String {
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, bytes)
}

#[derive(Clone, PartialEq, Eq, Hash, Deref, From, Display, DebugCustom)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct BlockReference([u8; 32]);

#[derive(Clone, Deref, From, Display, DebugCustom)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct BlockKey([u8; 32]);

type RKPair = (BlockReference, BlockKey);

#[derive(Deref, DerefMut, From, Display, DebugCustom)]
#[display(fmt = "{}", "display_base32(&self.0)")]
#[debug(fmt = "{}", "self")]
pub struct Block<const BS: usize>([u8; BS]);

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
                let arr: [u8; BS] = x.clone().try_into()
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Block has unexpected size"))?;
                Ok(arr.into())
            })
            .transpose()
    }
}

const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }

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

    pub fn urn(&self) -> String {
        format!("urn:erisx2:{}", &display_base32(&self.binary()))
    }
}

impl<const BS: usize> Block<BS> {
    pub(crate) fn reference(&self) -> BlockReference {
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
