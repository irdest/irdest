use crate::{
    crypto::CryEngineHandle,
    io::{
        cfg::Config,
        error::Result,
        wire::{ChunkList, Encrypted, FromEncrypted, FromReader},
        Chunk,
    },
};
use id::Identity;
use std::{collections::LinkedList, fs::File, path::PathBuf};

struct ChunkItem {
    id: Identity,
    chunk: Chunk,
}

/// The I/O representation of a record
///
/// At its core a record is a collection of chunks, as well as an
/// index of available chunks.  Each chunk has an ID which is passed
/// as metadata to data records to re-associate updated data to the
/// chunk that needs to be updated (or expanded).
pub(crate) struct Record {
    index: LinkedList<ChunkItem>,
    offset: PathBuf,
    idx_io: ChunkList,
    user: Identity,
    cry: CryEngineHandle,
    id: Identity,
    f: File,
}

impl Record {
    /// Create a new chunk and initialise it's chunk-file
    pub(crate) async fn new(
        offset: PathBuf,
        id: Identity,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let mut f = File::create(offset.join(id.to_string()))?;

        let e = Encrypted::from_reader(&mut f)?;
        let idx_io = ChunkList::from_encrypted(e, user, cry.clone()).await?;

        Ok(Self {
            index: LinkedList::new(),
            idx_io,
            offset,
            user,
            cry,
            id,
            f,
        })
    }

    /// Load an existing record from the record index file
    pub(crate) async fn load(
        cfg: &Config,
        offset: PathBuf,
        id: Identity,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let mut f = File::open(offset.join(id.to_string()))?;

        let e = Encrypted::from_reader(&mut f)?;
        let idx_io = ChunkList::from_encrypted(e, user, cry.clone()).await?;

        // For each chunk in the index, create a path, open the chunk
        // file and parse its contents.  Then add it to the index.
        let mut index = LinkedList::new();
        for id in idx_io.chunks() {
            let path = offset.join(id.to_string());
            let file = File::open(path)?;
            let chunk = Chunk::load(cfg, user, cry.clone(), file).await?;
            index.push_back(ChunkItem { id, chunk });
        }

        Ok(Self {
            index,
            offset,
            idx_io,
            user,
            cry,
            id,
            f,
        })
    }
}
