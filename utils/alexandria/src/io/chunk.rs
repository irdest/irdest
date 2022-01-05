//! Handling the reading and writing of a single chunk

use crate::{
    crypto::{CryEngine, CryEngineHandle},
    io::{
        error::Result,
        wire::{Encrypted, FromEncrypted, FromReader, Row, ToEncrypted, ToWriter},
        Config,
    },
};
use id::Identity;
use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
};

/// Panicing wrapper to get the size of a file
fn file_size(f: &File) -> u64 {
    f.metadata().expect("Failed to get file metadata").len() as u64
}

/// This test mostly ensures that the relationship between input
/// buffers and file metadata is correct.  This is as much a test of
/// the filesystem we are running on as checking the invariant of
/// misaligned file lengths in this code.
#[test]
fn test_file_size() {
    use std::io::Write;
    let data = (0..20).into_iter().map(|b| b as u8).collect::<Vec<u8>>();
    let mut tmp = tempfile::tempfile().unwrap();
    tmp.write_all(&data).unwrap();

    let len = file_size(&tmp);
    assert_eq!(len as usize, data.len());
}

/// Handle reads and writes to a single chunk
///
/// A chunk is a continous stream of encrypted data written to the
/// same file.  A record is a collection of different chunks.  If a
/// write operation produces data that overruns the chunk scope it
/// will be marked as `full` (checked via [`full()`](`Chunk::full()`))
///
/// This abstraction interacts with UNENCRYPTED DATA and runs
/// asynchronously to the rest of the database.  This abstraction
/// however does not care about _what_ kind of data is being handled.
pub(crate) struct Chunk {
    user: Identity,
    cry: CryEngineHandle,
    f: File,
    max_len: u64,
    cur_len: u64,
}

impl Chunk {
    /// Create a new, empty chunk
    pub(crate) fn new(cfg: &Config, user: Identity, cry: CryEngineHandle, f: File) -> Self {
        Self {
            user,
            cry,
            f,
            max_len: cfg.chunk_size,
            cur_len: 0,
        }
    }

    /// Load an existing chunk file into this lazy representation
    pub(crate) fn load(cfg: &Config, user: Identity, cry: CryEngineHandle, f: File) -> Self {
        Self {
            max_len: cfg.chunk_size,
            cur_len: file_size(&f),
            user,
            cry,
            f,
        }
    }

    /// Indicate whether this chunk should be considered "full"
    pub(crate) fn full(&self) -> bool {
        match (self.max_len, self.cur_len) {
            (m, c) if m >= c => true,     // obvious
            (m, c) if m + 64 > c => true, // smol grace section (todo: make configurable)
            _ => false,
        }
    }

    /// Append some data to this chunk
    ///
    /// Make sure to check whether the chunk is full afterwards!
    pub(crate) async fn append<T: ToEncrypted>(&mut self, data: T) -> Result<usize> {
        let e = data.to_encrypted(self.user, self.cry.clone()).await?;
        self.f.seek(SeekFrom::Start(self.cur_len))?;
        let len = e.to_writer(&mut self.f)? as u64;
        self.cur_len += len;
        Ok(len as usize)
    }

    /// Like `append()` but without internal encryption and encoding
    pub(crate) fn append_raw(&mut self, data: &Vec<u8>) -> Result<()> {
        self.f.seek(SeekFrom::Start(self.cur_len));
        self.f.write_all(data)?;
        self.cur_len += data.len() as u64;
        Ok(())
    }

    /// Move the seek point back to the start of the chunk
    pub(crate) fn seek_to_start(&mut self) -> Result<()> {
        self.f.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Move the seek point to the end of the last data section
    pub(crate) fn seek_to_end(&mut self) -> Result<()> {
        self.f.seek(SeekFrom::Start(self.cur_len))?;
        Ok(())
    }

    pub(crate) async fn next_section<T: FromEncrypted>(&mut self) -> Result<T> {
        let e = Encrypted::from_reader(&mut self.f)?;
        Ok(T::from_encrypted(e, self.user, self.cry.clone()).await?)
    }
}

#[async_std::test]
async fn one_chunk_write_read() {
    use crate::{
        crypto::{pkcry::keypair, CryEngine},
        io::wire::{RowData, RowHeader, TableHeader},
        meta::KeyStore,
    };
    use std::{
        io::{BufReader, BufWriter},
        sync::Arc,
    };

    ///////// SETUP ENVIRONMENT /////////

    let (pub_, sec_) = keypair();
    let keystore = KeyStore::new(pub_, sec_);
    let cry = CryEngine::new(Arc::clone(&keystore)).await;
    let cfg = Config::new();

    // Create a simple user with pub and priv key
    let user = Identity::random();
    let (upub, usec) = keypair();
    keystore.add_pair(user, upub, usec).await;

    ///////// START MAKING DATA /////////

    // Create a simple table header
    let mut a = TableHeader::new(
        vec!["name".into(), "age".into()],
        vec![vec![0, 1], vec![1, 0]],
    );
    a.add_row(); // add one row

    // Make a placeholder row-header
    let b = RowHeader::new(0, 0);

    // Create some row data and get its encrypted length
    let c = RowData::new(vec![
        vec!['b' as u8, 'o' as u8, 'b' as u8],
        vec![0, 0, 0, 64],
    ]);

    ///////// MAKE A NEW FILE /////////

    let mut f = tempfile::tempfile().unwrap();
    let mut chunk = Chunk::new(&cfg, user, cry.clone(), f);

    ///////// WRITE TO IT BIT BY BYTE /////////

    chunk.append(a).await.unwrap();
    Row::new(b, c)
        .append_group(user, cry.clone(), &mut chunk)
        .await
        .unwrap();

    ///////// READ FROM THE BEGINNING /////////

    chunk.seek_to_start();
    let a2: TableHeader = chunk.next_section().await.unwrap();
    let b2: RowHeader = chunk.next_section().await.unwrap();
    let c2: RowData = chunk.next_section().await.unwrap();
}
