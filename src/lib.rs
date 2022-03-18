use blake2::{Blake2b, Blake2bMac, Digest, digest::consts::U32, digest::Update, digest::KeyInit, digest::FixedOutput};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use thiserror::Error as ThisError;

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

impl From<BlockSize> for usize {
    fn from(block_size: BlockSize) -> usize {
        match block_size {
            BlockSize::_1K => 1024,
            BlockSize::_32K => 32 * 1024,
        }
    }
}

#[derive(Clone)]
pub struct BlockReference([u8; 32]);

impl From<[u8; 32]> for BlockReference {
    fn from(arr: [u8; 32]) -> BlockReference {
        BlockReference(arr)
    }
}

#[derive(Clone)]
pub struct BlockKey([u8; 32]);

impl From<[u8; 32]> for BlockKey {
    fn from(arr: [u8; 32]) -> BlockKey {
        BlockKey(arr)
    }
}

pub type RKPair = (BlockReference, BlockKey);

pub fn encode(content: &[u8], convergence_secret: &[u8; 32], block_size: BlockSize) -> (Vec<Vec<u8>>, RKPair) {
    let mut level = 0;

    let (mut blocks, mut rk_pairs) = split_content(content, convergence_secret, block_size);

    while rk_pairs.len() > 1 {
        let (mut level_blocks, new_rk_pairs) = collect_rk_pairs(rk_pairs, convergence_secret, block_size);
        rk_pairs = new_rk_pairs;

        blocks.append(&mut level_blocks);
        level += 1;
    }

    let root = rk_pairs[0].clone();

    (blocks, root)
}

fn split_content(content: &[u8], convergence_secret: &[u8; 32], block_size: BlockSize) -> (Vec<Vec<u8>>, Vec<RKPair>) {
    let mut blocks = vec![];
    let mut rk_pairs = vec![];

    let padded = {
        let mut buffer = content.to_vec();
        pad(&mut buffer, block_size);
        buffer
    };

    for content_block in padded.chunks_exact(block_size.into()) {
        let (encrypted_block, rk_pair) = encrypt_block(content_block, convergence_secret);
        blocks.push(encrypted_block);
        rk_pairs.push(rk_pair);
    }

    (blocks, rk_pairs)
}

fn collect_rk_pairs(mut input_rk_pairs: Vec<RKPair>, convergence_secret: &[u8; 32], block_size: BlockSize) -> (Vec<Vec<u8>>, Vec<RKPair>) {
    let arity = usize::from(block_size) / 64;

    let mut blocks = vec![];
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

        let (block, rk_pair) = encrypt_block(&node, convergence_secret);

        blocks.push(block);
        output_rk_pairs.push(rk_pair);
    }

    (blocks, output_rk_pairs)
}

fn pad(input: &mut Vec<u8>, block_size: BlockSize) {
    let block_size: usize = block_size.into();
    input.push(0x80);
    while input.len() % block_size != 0 {
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
        if old_len - input.len() > usize::from(block_size) {
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
