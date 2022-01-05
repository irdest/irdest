//! Wire encoding module
//!
//! This module wraps around an encoding library (currently protobuf)
//! to read and write data to disk.

mod encrypted;
pub(super) use encrypted::Encrypted;

mod chunk;
pub(super) use chunk::*;

mod table;
use id::Identity;
pub(super) use table::*;

mod traits;
pub(super) use traits::{FromEncrypted, FromReader, ToEncrypted, ToWriter};

use crate::io::error::Result;
use byteorder::{BigEndian, ByteOrder};
use std::io::{Read, Write};

/// First write the length as big-endian u64, then write the provided buffer
pub(self) fn write_with_length<T: Write>(t: &mut T, buf: &Vec<u8>) -> Result<usize> {
    let mut len = vec![0; 8];
    BigEndian::write_u64(&mut len, buf.len() as u64);
    t.write_all(&len)?;
    t.write_all(&buf)?;
    Ok(len.len() + buf.len())
}

/// First read a big-endian u64, then read the number of bytes
pub(self) fn read_with_length<T: Read>(t: &mut T) -> Result<Vec<u8>> {
    let mut len_buf = vec![0; 8];
    t.read_exact(&mut len_buf)?;
    let len = BigEndian::read_u64(&len_buf);

    let mut vec = vec![0; len as usize]; // FIXME: this might break on 32bit systems
    t.read_exact(&mut vec)?;
    Ok(vec)
}

/// This test creates a simple stream of types that must be read in
/// sequence with proper delimitation in order to maintain separation
/// of scopes.
///
/// This test uses un-encrypted data types (`table::*` data types) to
/// make things easier and MUST NOT be seen as an example of how to
/// write data in alexandria in general!
#[test]
fn write_and_read() {
    use std::io::{BufReader, BufWriter};

    // Create a table header with fields "name" and "age" and column
    // types "0, 1" and "1, 0" (not real types)
    let a = TableHeader::new(
        vec!["name".into(), "age".into()],
        vec![vec![0, 1], vec![1, 0]],
    );

    // Create a Row data section first so we know how long it is.  In
    // actuality the length of the data section is determined by the
    // ENCRYPTED payload length.
    let c = RowData::new(vec![
        vec!['b' as u8, 'o' as u8, 'b' as u8],
        vec![0, 0, 0, 64],
    ]);
    let mut c_buf = vec![];
    c.to_writer(&mut c_buf).unwrap();
    let len = c_buf.len();

    let b = RowHeader::new(0, len as u64);

    let mut buffer = vec![];
    let mut writer = BufWriter::new(buffer);

    a.to_writer(&mut writer).unwrap();
    b.to_writer(&mut writer).unwrap();
    c.to_writer(&mut writer).unwrap();

    ///////// INTERMISSION /////////

    let buffer = writer.into_inner().unwrap();
    assert_eq!(buffer.len(), 56);

    ///////////////////////////////

    let mut r = BufReader::new(buffer.as_slice());
    let a2 = TableHeader::from_reader(&mut r).unwrap();
    let b2 = RowHeader::from_reader(&mut r).unwrap();
    let c2 = RowData::from_reader(&mut r).unwrap();

    assert_eq!(a, a2);
    assert_eq!(b, b2);
    assert_eq!(c, c2);
}

#[async_std::test]
async fn single_encrypted_cycle() {
    use crate::{
        crypto::{pkcry::keypair, CryEngine},
        io::wire::{Encrypted, ToEncrypted, ToWriter},
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

    // Create a simple user with pub and priv key
    let user = Identity::random();
    let (upub, usec) = keypair();
    keystore.add_pair(user, upub, usec).await;

    ///////// START MAKING DATA /////////

    let chunk = ChunkHeader::new(1024);
    let enc = chunk.to_encrypted(user, cry.clone()).await.unwrap();

    let chunk2 = ChunkHeader::from_encrypted(enc, user, cry).await.unwrap();
    assert_eq!(chunk, chunk2);
}

/// A more realistic example of how to write and read
#[async_std::test]
async fn write_and_read_encrypted() {
    use crate::{
        crypto::{pkcry::keypair, CryEngine},
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

    // Create some row data and get its encrypted length
    let c = RowData::new(vec![
        vec!['b' as u8, 'o' as u8, 'b' as u8],
        vec![0, 0, 0, 64],
    ]);
    // let c_enc = c.to_encrypted(user, cry.clone()).await.unwrap();
    // let mut c_buf = vec![];
    // c_enc.to_writer(&mut c_buf).unwrap();
    // let row_len = c_buf.len();

    // Create the row data and save the data section length
    let b = RowHeader::new(0, 0 as u64);

    ///////// WRITE DATA TO A STREAM /////////

    let mut buffer = vec![];
    let mut writer = BufWriter::new(buffer);

    a.to_encrypted(user, cry.clone())
        .await
        .unwrap()
        .to_writer(&mut writer)
        .unwrap();

    b.to_encrypted(user, cry.clone())
        .await
        .unwrap()
        .to_writer(&mut writer)
        .unwrap();

    c.to_encrypted(user, cry.clone())
        .await
        .unwrap()
        .to_writer(&mut writer)
        .unwrap();

    ///////// INTERMISSION /////////

    let buffer = writer.into_inner().unwrap();
    assert_eq!(buffer.len(), 212);

    ///////////////////////////////

    ///////// READ ENCRYPTED DATA FIRST /////////

    let mut r = BufReader::new(buffer.as_slice());
    let a_enc = Encrypted::from_reader(&mut r).unwrap();
    let b_enc = Encrypted::from_reader(&mut r).unwrap();
    let c_enc = Encrypted::from_reader(&mut r).unwrap();

    ///////// DECRYPT DATA AGAIN /////////

    let a2 = TableHeader::from_encrypted(a_enc, user, cry.clone())
        .await
        .unwrap();
    let b2 = RowHeader::from_encrypted(b_enc, user, cry.clone())
        .await
        .unwrap();
    let c2 = RowData::from_encrypted(c_enc, user, cry.clone())
        .await
        .unwrap();

    assert_eq!(a, a2);
    assert_eq!(b, b2);
    assert_eq!(c, c2);
}
