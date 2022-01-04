//! Wire encoding module
//!
//! This module wraps around an encoding library (currently protobuf)
//! to read and write data to disk.

mod encrypted;

pub use encrypted::*;

mod chunk;
pub use chunk::*;

mod table;
pub use table::*;

use crate::io::error::Result;
use byteorder::{BigEndian, ByteOrder};
use std::io::{Read, Write};

/// First write the length as big-endian u64, then write the provided buffer
pub(self) fn write_with_length<T: Write>(t: &mut T, buf: &Vec<u8>) -> Result<()> {
    let mut len = vec![0; 8];
    BigEndian::write_u64(&mut len, buf.len() as u64);
    t.write_all(&len)?;
    t.write_all(&buf)?;
    Ok(())
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
