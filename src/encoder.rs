use blake2::{Blake2bMac, digest::consts::U32, digest::Update, digest::KeyInit, digest::FixedOutput};
use crate::{RKPair, ReadCapability, BlockSize, BlockStorageWrite, BlockKey, chacha20, block_reference};

fn pad(input: &mut Vec<u8>, block_size: BlockSize) {
    input.push(0x80);
    while input.len() % *block_size != 0 {
        input.push(0x0);
    }
}

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

pub struct Encoder<'a, S: BlockStorageWrite> {
    pub convergence_secret: [u8; 32],
    pub block_size: BlockSize,
    pub block_storage: &'a mut S,
}

pub async fn encode<S: BlockStorageWrite>(content: &[u8], convergence_secret: &[u8; 32], block_size: BlockSize, block_storage: &mut S) -> std::io::Result<ReadCapability> {
    let mut encoder = Encoder {
        convergence_secret: convergence_secret.clone(),
        block_size,
        block_storage,
    };
    encoder.encode(content).await
}

impl<'a, S: BlockStorageWrite> Encoder<'a, S> {
    pub async fn encode(&mut self, content: &[u8]) -> std::io::Result<ReadCapability> {
        let mut level = 0;
        let mut rk_pairs = self.split_content(content).await?;

        while rk_pairs.len() > 1 {
            let new_rk_pairs = self.collect_rk_pairs(rk_pairs).await?;
            rk_pairs = new_rk_pairs;
            level += 1;
        }

        let root = rk_pairs.remove(0);
        Ok(ReadCapability::from_rk_pair(root, level, self.block_size))
    }

    async fn split_content(&mut self, content: &[u8]) -> std::io::Result<Vec<RKPair>> {
        let mut rk_pairs = vec![];

        let mut padded = {
            let mut buffer = content.to_vec();
            pad(&mut buffer, self.block_size);
            buffer
        };

        for mut block in padded.chunks_exact_mut(*self.block_size) {
            let rk_pair = encrypt_block(&mut block, &self.convergence_secret);
            self.block_storage.store(block).await?;
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
