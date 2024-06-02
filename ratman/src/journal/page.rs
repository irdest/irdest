use async_eris::{Block, BlockReference, BlockStorage};
use async_trait::async_trait;
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use libratman::{
    frame::{FrameGenerator, FrameParser},
    EncodingError, RatmanError, Result,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::Result as IoResult,
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::storage::block::StorageBlock;

use super::event::BlockEvent;

/// Represent a single logical page in the storage journal
///
/// A journal page has an associated Rust type which it will serialize and
/// deserialize for storage. Each page can be configured with a custom block
/// size if that is desired.
pub struct JournalPage<T: Serialize + DeserializeOwned>(pub PartitionHandle, pub PhantomData<T>);

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
        let bin_data = self.0.get(key)?.unwrap();
        Ok(bincode::deserialize(&*bin_data)?)
    }
}

#[async_trait]
impl<const L: usize> BlockStorage<L> for JournalPage<BlockEvent> {
    async fn store(&mut self, block: &Block<L>) -> IoResult<()> {
        self.insert(
            block.reference().to_string(),
            &BlockEvent::Insert(StorageBlock::from_block(block)),
        )?;
        Ok(())
    }

    async fn fetch(&self, reference: &BlockReference) -> IoResult<Option<Block<L>>> {
        match self.get(&reference.to_string()) {
            Err(RatmanError::Io(io)) => Err(io),
            Ok(BlockEvent::Insert(storage_block)) => Ok(Some(storage_block.to_block())),
            _ => Ok(None),
        }
    }
}

/// Allow serialising of non-serde types
///
/// Because serde doesn't allow serialising arrays larger than 32 items and many
/// frames in Irdest use 64 byte arrays for metadata side channels we need to
/// wrap these types in our own serde type.
///
/// A SerdeFrameType can be constructed via `From<FrameGenerator>` and
/// deserialised via `to_frametype()` which invokes `FrameParser` (both from
/// libratman).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerdeFrameType<T>(Vec<u8>, PhantomData<T>);

impl<T: FrameGenerator> From<T> for SerdeFrameType<T> {
    fn from(input: T) -> SerdeFrameType<T> {
        let mut frame_buf = vec![];
        input.generate(&mut frame_buf);

        SerdeFrameType(frame_buf, PhantomData)
    }
}

impl<T: FrameParser> SerdeFrameType<T> {
    pub fn to_frametype(&self) -> Result<T> {
        match T::parse(&self.0) {
            Ok(t) => Ok(t),
            Err(e) => RatmanError::Encoding(EncodingError::from(e)),
        }
    }
}
