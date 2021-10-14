use crate::io::{
    chunk::{ChunkError, DataChunk, Result},
    Encode,
};
use std::io::{Seek, SeekFrom, Write};

type WriteTuple = (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>);

/// Prepend the length of a vector
fn len_vec(mut v: Vec<u8>) -> Vec<u8> {
    let mut buf = (v.len() as u64).to_le_bytes().to_vec();
    buf.append(&mut v);
    buf
}

/// Encode a piece of data with its length prepended
fn encode<E: Encode>(e: &E) -> Result<Vec<u8>> {
    Ok(e.encode().map(|mut vec| len_vec(vec))?)
}

/// Append a full set of chunks to an open file
pub(crate) fn append_chunk<F: Seek + Write>(
    file: &mut F,
    start: u64,
    chunks: Vec<DataChunk>,
) -> Result<()> {
    file.seek(SeekFrom::Start(start))?;

    // First encode all the chunk data into tuples to ensure we don't
    // half-write non-encodable information
    let tuples = chunks
        .into_iter()
        .map(
            |DataChunk {
                 nonce,
                 len,
                 head,
                 data,
             }| {
                let len = encode(&len)?;
                let head = encode(&head)?;

                Ok((len_vec(nonce), len, head, len_vec(data)))
            },
        )
        .collect::<Result<Vec<WriteTuple>>>()?;

    // Write out the chunks in stages

    // FIXME: currently "len" and "head" are not encrypted which may
    // reveal information about padding length to an attacker
    for (nonce, len, head, data) in tuples {
        file.write_all(&nonce)?;
        file.write_all(&len)?;
        file.write_all(&head)?;
        file.write_all(&data)?;
    }

    // Then sync!
    file.flush()?;

    Ok(())
}
