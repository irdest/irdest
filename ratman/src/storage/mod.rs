//! This module handles several storage engines.  The backing database is fjall, a
//!
//! - Block storage: keep track of full blocks
//!
//! - Frame storage: keep track of in-flight frames that don't fully assemble a
//! block (yet)
//!
//! - Peer metadata: persistent routing tables
//!
//! -

use async_eris::{Block, BlockReference, BlockStorage};
use async_trait::async_trait;
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use libratman::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod addrs;
pub mod block;
pub mod client;
pub mod journal;

/// Represent a single logical page in the storage journal
///
/// A journal page has an associated Rust type which it will serialize and
/// deserialize for storage. Each page can be configured with a custom block
/// size if that is desired.
pub struct JournalPage<T: Serialize + DeserializeOwned>(PartitionHandle, PhantomData<T>);

impl<T: Serialize + DeserializeOwned> JournalPage<T> {
    pub fn new(keyspace: &Keyspace, name: &str, block_size: Option<u32>) -> Result<Self> {
        let inner = keyspace.open_partition(
            name,
            match block_size {
                Some(bs) => PartitionCreateOptions::default().block_size(bs),
                None => PartitionCreateOptions::default(),
            },
        )?;
        Ok(Self(inner, PhantomData))
    }

    pub fn insert(&self, key: String, value: &T) -> Result<()> {
        let bin = bincode::serialize(value)?;
        self.0.insert(key, bin);
        Ok(())
    }

    pub fn get(&self, key: &String) -> Result<T> {
        let bin_data = self.0.get(key)?;
        Ok(bincode::deserialize(&bin_data)?)
    }
}

// #[async_trait]
// impl<const L: usize> BlockStorage<L> for JournalPage<Block<L>>
// {
//     async fn store(&mut self, block: &Block<L>) -> std::io::Result<()> {
//         self.insert(block.reference(), block.as_slice())?;
//         Ok(())
//     }

//     async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Block<L>>> {
//         self.get(&reference.to_string())
//     }
// }
