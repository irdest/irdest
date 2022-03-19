use blake2::{Blake2bMac, digest::consts::U32, digest::Update, digest::KeyInit, digest::FixedOutput};
use crate::{RKPair, ReadCapability, BlockStorage, BlockKey, chacha20, block_reference};
use futures_lite::io::{AsyncRead, AsyncReadExt};

fn encrypt_block(block: &mut [u8], convergence_secret: &[u8; 32]) -> RKPair {
    let key = block_key(block, convergence_secret);
    chacha20(block, &key);
    let reference = block_reference(&block);

    (reference, key)
}

fn block_key(input: &[u8], convergence_secret: &[u8; 32]) -> BlockKey {
    let mut hasher = Blake2bMac::<U32>::new_from_slice(convergence_secret).unwrap();
    Update::update(&mut hasher, input);
    BlockKey(hasher.finalize_fixed().into())
}

pub struct Encoder<'a, S: BlockStorage, const BS: usize> {
    pub convergence_secret: [u8; 32],
    pub block_storage: &'a mut S,
}

#[derive(Clone, Copy, Debug)]
pub enum BlockSize {
    _1K,
    _32K,
}

pub async fn encode<S: BlockStorage, R: AsyncRead + Unpin>(content: &mut R, convergence_secret: &[u8; 32], block_size: BlockSize, block_storage: &mut S) -> std::io::Result<ReadCapability> {
    match block_size {
        BlockSize::_1K => (Encoder::<S, 1024> {
            convergence_secret: convergence_secret.clone(),
            block_storage,
        }).encode(content).await,
        BlockSize::_32K => (Encoder::<S, {32 * 1024}> {
            convergence_secret: convergence_secret.clone(),
            block_storage,
        }).encode(content).await,
    }
}

impl<'a, S: BlockStorage, const BS: usize> Encoder<'a, S, BS> {
    pub async fn encode<R: AsyncRead + Unpin>(&mut self, content: &mut R) -> std::io::Result<ReadCapability> {
        let mut level = 0;
        let mut rk_pairs = self.split_content(content).await?;

        while rk_pairs.len() > 1 {
            let new_rk_pairs = self.collect_rk_pairs(rk_pairs).await?;
            rk_pairs = new_rk_pairs;
            level += 1;
        }

        let root = rk_pairs.remove(0);
        Ok(ReadCapability::from_rk_pair(root, level, BS))
    }

    async fn split_content<R: AsyncRead + Unpin>(&mut self, content: &mut R) -> std::io::Result<Vec<RKPair>> {
        let mut rk_pairs = vec![];
        let mut buf = [0u8; BS];
        let mut pos;
        loop {
            pos = 0;
            while pos < BS {
                match content.read(&mut buf[pos..]).await? {
                    0 => break,
                    n => {
                        pos += n;
                    },
                };
            }
            if pos != BS {
                buf[pos..].fill(0);
                buf[pos] = 0x80;
            }

            let rk_pair = encrypt_block(&mut buf, &self.convergence_secret);
            self.block_storage.store(&buf).await?;
            rk_pairs.push(rk_pair);
            if pos != BS { break; };
        }

        Ok(rk_pairs)
    }

    async fn collect_rk_pairs(&mut self, mut input_rk_pairs: Vec<RKPair>) -> std::io::Result<Vec<RKPair>> {
        let arity = BS / 64;

        let mut output_rk_pairs = vec![];

        while input_rk_pairs.len() % arity != 0 {
            input_rk_pairs.push(([0; 32].into(), [0; 32].into()));
        }

        for rk_pairs_for_node in input_rk_pairs.chunks_exact(arity) {
            let mut node = {
                let mut buffer = vec![];
                for pair in rk_pairs_for_node {
                    buffer.extend_from_slice(&pair.0.0);
                    buffer.extend_from_slice(&pair.1.0);
                }
                buffer
            };

            let rk_pair = encrypt_block(&mut node, &self.convergence_secret);

            self.block_storage.store(&node).await?;
            output_rk_pairs.push(rk_pair);
        }

        Ok(output_rk_pairs)
    }
}
