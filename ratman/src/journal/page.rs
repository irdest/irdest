// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{journal::types::BlockData, storage::block::StorageBlock};
use async_eris::{Block, BlockReference, BlockStorage};
use async_trait::async_trait;
use fjall::{Keyspace, PartitionCreateOptions, PartitionHandle};
use libratman::{
    frame::{FrameGenerator, FrameParser},
    EncodingError, RatmanError, Result,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::Result as IoResult, marker::PhantomData};

/// Represent a single logical page in the fjall database
///
/// A cache page has an associated Rust type which it will serialize and
/// deserialize for storage. Each page can be configured with a custom block
/// size if that is desired.
pub struct CachePage<T: Serialize + DeserializeOwned>(pub PartitionHandle, pub PhantomData<T>);

impl<T: Serialize + DeserializeOwned> CachePage<T> {
    pub fn insert(&self, key: String, value: &T) -> Result<()> {
        let bin = bincode::serialize(value)?;
        self.0.insert(key, bin)?;
        Ok(())
    }

    pub fn remove(&self, key: String) -> Result<()> {
        self.0.remove(key)?;
        Ok(())
    }

    pub fn get(&self, key: &String) -> Result<Option<T>> {
        Ok(self
            .0
            .get(key)?
            .map(|bin_data| bincode::deserialize(&*bin_data).expect("failed deserialising")))
    }

    /// Perform a prefix key search and filter out invalid entries
    pub fn prefix<'key>(
        &'key self,
        prefix: &'key String,
    ) -> impl DoubleEndedIterator<Item = (String, T)> + 'key {
        self.0
            .prefix(prefix)
            // filter out read read failures
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            // then deserialise the data
            .map(|(item_key, item_data)| {
                (
                    String::from_utf8(item_key.to_vec()),
                    bincode::deserialize(&*item_data),
                )
            })
            // then filter out encoding failures
            .filter(|(x, y)| x.is_ok() && y.is_ok())
            .map(|(x, y)| (x.unwrap(), y.unwrap()))
    }

    /// Get an iterator over all valid entries on this page and their keys
    pub fn iter<'page>(&'page self) -> impl DoubleEndedIterator<Item = (String, T)> + 'page {
        self.0.iter().filter_map(|item| match item {
            Ok((key, val)) => Some((
                String::from_utf8(key.to_vec()).unwrap(),
                bincode::deserialize(&*val).unwrap(),
            )),
            Err(_) => None,
        })
    }
}

#[async_trait]
impl<const L: usize> BlockStorage<L> for CachePage<BlockData> {
    async fn store(&self, block: &Block<L>) -> IoResult<()> {
        self.insert(
            block.reference().to_string(),
            &BlockData {
                // We can unwrap here because the blocks were previously
                // verified to be valid
                data: StorageBlock::reconstruct(block.as_slice()).unwrap(),
                valid: true,
            },
        )?;
        Ok(())
    }

    async fn fetch(&self, reference: &BlockReference) -> IoResult<Option<Block<L>>> {
        match self.get(&reference.to_string()) {
            Err(RatmanError::Io(io)) => Err(io),
            Ok(Some(BlockData { data, valid })) if valid => Ok(Some(data.to_block())),
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerdeFrameType<T>(Vec<u8>, PhantomData<T>);

impl<T: FrameGenerator> From<T> for SerdeFrameType<T> {
    fn from(input: T) -> SerdeFrameType<T> {
        let mut frame_buf = vec![];
        input.generate(&mut frame_buf).unwrap();
        SerdeFrameType(frame_buf, PhantomData)
    }
}

impl<T: FrameParser<Output = Result<T>>> SerdeFrameType<T> {
    pub fn maybe_inner(&self) -> Result<T> {
        match T::parse(&self.0) {
            Ok((_, t)) => t,
            Err(e) => Err(RatmanError::Encoding(EncodingError::from(e))),
        }
    }
}

/// A simple cache page which keeps track of the existence of values
pub struct JournalCache<T: AsRef<[u8]>>(pub PartitionHandle, pub PhantomData<T>);

impl<T: AsRef<[u8]>> JournalCache<T> {
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

    pub fn insert(&self, value: &T) -> Result<()> {
        self.0.insert(value, &[true as u8])?;
        Ok(())
    }

    pub fn get(&self, key: &T) -> Result<bool> {
        Ok(self.0.get(key)?.is_some())
    }
}
