use blake2::{Blake2b, Blake2bMac, Digest, digest::consts::U32, digest::Update, digest::KeyInit, digest::FixedOutput};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use thiserror::Error as ThisError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid padding")]
    Padding,
}
pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Clone, Copy)]
pub enum BlockSize {
    _1K,
    _32K,
}

impl Deref for BlockSize {
    type Target = usize;
    fn deref(&self) -> &usize {
        match self {
            BlockSize::_1K => &1024,
            BlockSize::_32K => &(32 * 1024),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

pub type RKPair = (BlockReference, BlockKey);

#[async_trait]
pub trait BlockStorageRead {
    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Vec<u8>>>;
}

#[async_trait]
pub trait BlockStorageWrite {
    async fn store(&mut self, block: &[u8]) -> std::io::Result<()>;
}

#[async_trait]
impl BlockStorageRead for HashMap<BlockReference, Vec<u8>> {
    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Vec<u8>>> {
        Ok(self.get(reference).cloned())
    }
}

#[async_trait]
impl BlockStorageWrite for HashMap<BlockReference, Vec<u8>> {
    async fn store(&mut self, block: &[u8]) -> std::io::Result<()> {
        self.insert(block_reference(block), block.to_vec());
        Ok(())
    }
}

pub struct Encoder<'a, S: BlockStorageWrite> {
    convergence_secret: [u8; 32],
    block_size: BlockSize,
    block_storage: &'a mut S,
}

pub async fn encode<S: BlockStorageWrite>(content: &[u8], convergence_secret: &[u8; 32], block_size: BlockSize, block_storage: &mut S) -> std::io::Result<RKPair> {
    let mut encoder = Encoder {
        convergence_secret: convergence_secret.clone(),
        block_size,
        block_storage,
    };
    encoder.encode(content).await
}

impl<'a, S: BlockStorageWrite> Encoder<'a, S> {
    pub async fn encode(&mut self, content: &[u8]) -> std::io::Result<RKPair> {
        let mut level = 0;
        let mut rk_pairs = self.split_content(content).await?;

        while rk_pairs.len() > 1 {
            let new_rk_pairs = self.collect_rk_pairs(rk_pairs).await?;
            rk_pairs = new_rk_pairs;
            level += 1;
        }

        let root = rk_pairs.remove(0);
        Ok(root)
    }

    async fn split_content(&mut self, content: &[u8]) -> std::io::Result<Vec<RKPair>> {
        let mut rk_pairs = vec![];

        let padded = {
            let mut buffer = content.to_vec();
            pad(&mut buffer, self.block_size);
            buffer
        };

        for content_block in padded.chunks_exact(*self.block_size) {
            let (encrypted_block, rk_pair) = encrypt_block(content_block, &self.convergence_secret);
            self.block_storage.store(&encrypted_block).await?;
            rk_pairs.push(rk_pair);
        }

        Ok(rk_pairs)
    }

    async fn collect_rk_pairs(&mut self, mut input_rk_pairs: Vec<RKPair>) -> std::io::Result<Vec<RKPair>> {
        let arity = *self.block_size / 64;

        let mut output_rk_pairs = vec![];

        while input_rk_pairs.len() % arity != 0 {
            input_rk_pairs.push(([0; 32].into(), [0; 32].into()));
        }

        for rk_pairs_for_node in input_rk_pairs.chunks_exact(arity) {
            let node = {
                let mut buffer = vec![];
                for pair in rk_pairs_for_node {
                    buffer.extend_from_slice(&pair.0.0);
                    buffer.extend_from_slice(&pair.1.0);
                }
                buffer
            };

            let (block, rk_pair) = encrypt_block(&node, &self.convergence_secret);

            self.block_storage.store(&block).await?;
            output_rk_pairs.push(rk_pair);
        }

        Ok(output_rk_pairs)
    }
}

fn pad(input: &mut Vec<u8>, block_size: BlockSize) {
    input.push(0x80);
    while input.len() % *block_size != 0 {
        input.push(0x0);
    }
}

fn unpad(input: &mut Vec<u8>, block_size: BlockSize) -> Result {
    let old_len = input.len();
    loop {
        match input.pop() {
            Some(0) => (),
            Some(0x80) => return Ok(()),
            _ => return Err(Error::Padding),
        }
        if old_len - input.len() > *block_size {
            return Err(Error::Padding);
        }
    }
}

fn block_key(input: &[u8], convergence_secret: &[u8; 32]) -> BlockKey {
    let mut hasher = Blake2bMac::<U32>::new_from_slice(convergence_secret).unwrap();
    Update::update(&mut hasher, input);
    BlockKey(hasher.finalize_fixed().into())
}

fn block_reference(input: &[u8]) -> BlockReference {
    let mut hasher = Blake2b::<U32>::new();
    Digest::update(&mut hasher, &input);
    BlockReference(hasher.finalize().into())
}

fn encrypt_block(input: &[u8], convergence_secret: &[u8; 32]) -> (Vec<u8>, RKPair) {
    let key = block_key(input, convergence_secret);
    let encrypted_block = {
        let nonce = [0; 12];
        let mut cipher = ChaCha20::new(&key.0.into(), &nonce.into());
        let mut buffer = input.to_vec();
        cipher.apply_keystream(&mut buffer);
        buffer
    };
    let reference = block_reference(&encrypted_block);

    (encrypted_block, (reference, key))
}
