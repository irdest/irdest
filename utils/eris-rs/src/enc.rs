// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{Block, BlockKey, BlockStorage, RKPair, ReadCapability};
use blake2::{
    digest::consts::U32, digest::FixedOutput, digest::KeyInit, digest::Update, Blake2bMac,
};
use futures_lite::io::{AsyncRead, AsyncReadExt};

impl<const BS: usize> Block<BS> {
    fn encrypt(&mut self, convergence_secret: &[u8; 32]) -> RKPair {
        let key = self.derive_key(convergence_secret);
        self.chacha20(&key);
        let reference = self.reference();
        (reference, key)
    }

    fn derive_key(&self, convergence_secret: &[u8; 32]) -> BlockKey {
        let mut hasher = Blake2bMac::<U32>::new_from_slice(convergence_secret).unwrap();
        Update::update(&mut hasher, &**self);
        BlockKey(hasher.finalize_fixed().into())
    }
}

pub struct Encoder<'a, S: BlockStorage<BS>, const BS: usize> {
    pub convergence_secret: [u8; 32],
    pub block_storage: &'a S,
}

/// Supported block sizes by this implementation
#[derive(thiserror::Error, Clone, Copy, Debug)]
pub enum BlockSize {
    #[error("1kB")]
    _1K,
    #[error("32kB")]
    _32K,
}

/// Encode an async read stream into a set of blocks
///
/// Blocks are asynchronously streamed into the `block_storage` (which
/// may simply be in-memory, but could be on-disk for larger payloads).
///
/// This function returns a `ReadCapability`, which acts as a manifest
/// for the generated blocks.
pub async fn encode<S: BlockStorage<1024> + BlockStorage<32768>, R: AsyncRead + Unpin>(
    content: &mut R,
    convergence_secret: &[u8; 32],
    block_size: BlockSize,
    block_storage: &S,
) -> std::io::Result<ReadCapability> {
    match block_size {
        BlockSize::_1K => {
            encode_const::<_, _, 1024>(content, convergence_secret, block_storage).await
        }
        BlockSize::_32K => {
            encode_const::<_, _, 32768>(content, convergence_secret, block_storage).await
        }
    }
}

pub async fn encode_const<S: BlockStorage<BS>, R: AsyncRead + Unpin, const BS: usize>(
    content: &mut R,
    convergence_secret: &[u8; 32],
    block_storage: &S,
) -> std::io::Result<ReadCapability> {
    let mut encoder = Encoder::<S, BS> {
        convergence_secret: convergence_secret.clone(),
        block_storage,
    };
    encoder.encode(content).await
}

impl<'a, S: BlockStorage<BS>, const BS: usize> Encoder<'a, S, BS> {
    pub async fn encode<R: AsyncRead + Unpin>(
        &mut self,
        content: &mut R,
    ) -> std::io::Result<ReadCapability> {
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

    async fn split_content<R: AsyncRead + Unpin>(
        &mut self,
        content: &mut R,
    ) -> std::io::Result<Vec<RKPair>> {
        let mut rk_pairs = vec![];
        let mut buf = Block([0u8; BS]);
        let mut pos;
        loop {
            pos = 0;
            while pos < BS {
                match content.read(&mut buf[pos..]).await? {
                    0 => break,
                    n => {
                        pos += n;
                    }
                };
            }
            if pos != BS {
                buf[pos..].fill(0);
                buf[pos] = 0x80;
            }

            let rk_pair = buf.encrypt(&self.convergence_secret);
            self.block_storage.store(&buf).await?;
            rk_pairs.push(rk_pair);
            if pos != BS {
                break;
            };
        }

        Ok(rk_pairs)
    }

    async fn collect_rk_pairs(
        &mut self,
        input_rk_pairs: Vec<RKPair>,
    ) -> std::io::Result<Vec<RKPair>> {
        let arity = BS / 64;

        let mut output_rk_pairs = vec![];

        for rk_pairs_for_node in input_rk_pairs.chunks(arity) {
            let mut node = Block([0u8; BS]);
            for (x, pair) in rk_pairs_for_node.iter().enumerate() {
                node[64 * x..64 * x + 32].copy_from_slice(&pair.0 .0);
                node[64 * x + 32..64 * x + 64].copy_from_slice(&pair.1 .0);
            }

            let rk_pair = node.encrypt(&self.convergence_secret);

            self.block_storage.store(&node).await?;
            output_rk_pairs.push(rk_pair);
        }

        Ok(output_rk_pairs)
    }
}
