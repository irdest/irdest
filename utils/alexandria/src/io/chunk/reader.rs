use crate::{
    crypto::pkcry::{PubKey, SecKey},
    io::{
        chunk::{ChunkError, DataChunk, Result},
        Decode,
    },
};
use async_std::{
    channel::{self, Sender},
    sync::Arc,
};
use std::io::{Read, Seek, SeekFrom};

/// A reader to read a series of chunks from a record
///
/// This type doesn't get access to the raw file path because it is
/// not responsible for reading the record header.  Instead it is
/// created for a particular stream of data to read chunks, decode
/// them and loading them into a channel for the rest of alexandria to
/// parse.
pub(crate) struct ChunkReader<F>
where
    F: Seek + Read,
{
    io: F,
    pk: Arc<PubKey>,
    sk: Arc<SecKey>,
    tx: Sender<DataChunk>,
    seek: u64,
}

/// Read the length of a field first, then read the field
fn read_vec(f: &mut impl Read) -> Result<Vec<u8>> {
    let mut len_buf = [0; 8];
    f.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf);

    let mut data_buf = Vec::with_capacity(len as usize);
    f.read_exact(&mut data_buf)?;
    Ok(data_buf)
}

impl<F> ChunkReader<F>
where
    F: Seek + Read,
{
    /// Create a new ChunkReader for a stream
    pub fn create(io: F, pk: Arc<PubKey>, sk: Arc<SecKey>, tx: Sender<DataChunk>) -> Self {
        Self {
            io,
            pk,
            sk,
            tx,
            seek: 0,
        }
    }

    /// Read a single chunk into the sender
    pub fn read(&mut self) -> Result<()> {
        let nonce = read_vec(&mut self.io)?;
        let len = read_vec(&mut self.io).and_then(|v| u64::decode(&v).map_err(Into::into))?;
        let head = read_vec(&mut self.io).and_then(|v| u64::decode(&v).map_err(Into::into))?;
        let data = read_vec(&mut self.io)?;

        self.tx.send(DataChunk {
            nonce,
            len,
            head,
            data,
        });

        Ok(())
    }
}
