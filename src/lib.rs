use blake2::{Blake2b, Digest, digest::consts::U32};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::Deref;

mod enc;
pub use enc::{Encoder, encode, encode_const, BlockSize};
mod dec;
pub use dec::{decode, decode_const, Error, Result};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlockReference([u8; 32]);

impl From<[u8; 32]> for BlockReference {
    fn from(arr: [u8; 32]) -> BlockReference {
        BlockReference(arr)
    }
}

impl Deref for BlockReference {
    type Target = [u8; 32];
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for BlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**self))
    }
}

impl std::fmt::Debug for BlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct BlockKey([u8; 32]);

impl From<[u8; 32]> for BlockKey {
    fn from(arr: [u8; 32]) -> BlockKey {
        BlockKey(arr)
    }
}

impl Deref for BlockKey {
    type Target = [u8; 32];
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for BlockKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**self))
    }
}

impl std::fmt::Debug for BlockKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

type RKPair = (BlockReference, BlockKey);

#[async_trait]
pub trait BlockStorage<const BS: usize> {
    async fn store(&mut self, block: &[u8; BS]) -> std::io::Result<()>;
    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<[u8; BS]>>;
}

pub type MemoryStorage = HashMap<BlockReference, Vec<u8>>;

#[async_trait]
impl<const BS: usize> BlockStorage<BS> for MemoryStorage {
    async fn store(&mut self, block: &[u8; BS]) -> std::io::Result<()> {
        self.insert(block_reference(block), block.to_vec());
        Ok(())
    }

    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<[u8; BS]>> {
        let arr = self.get(reference).map(|x| {
            x.clone().try_into()
        }).transpose().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Block has unexpected size"))?;
        Ok(arr)
    }
}

#[derive(Clone, Debug)]
pub struct ReadCapability {
    pub root_reference: BlockReference,
    pub root_key: BlockKey,
    pub level: usize,
    pub block_size: usize,
}

impl ReadCapability {
    pub(crate) fn from_rk_pair(rk_pair: RKPair, level: usize, block_size: usize) -> ReadCapability {
        ReadCapability {
            root_reference: rk_pair.0,
            root_key: rk_pair.1,
            level,
            block_size,
        }
    }
}

pub(crate) fn block_reference(input: &[u8]) -> BlockReference {
    let mut hasher = Blake2b::<U32>::new();
    Digest::update(&mut hasher, &input);
    BlockReference(hasher.finalize().into())
}

pub(crate) fn chacha20(data: &mut [u8], key: &BlockKey) {
    let nonce = [0; 12];
    let mut cipher = ChaCha20::new(&(**key).into(), &nonce.into());
    cipher.apply_keystream(data);
}
