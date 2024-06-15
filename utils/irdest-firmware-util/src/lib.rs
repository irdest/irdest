// TODO: can we set no_std only when the "std" feature is not set?
// #[cfg(not(feature = "std"))] // <-- this doesn't seem to work?
// #![no_std]
#![allow(warnings)]
use libratman::{
    frame::{
        carrier::{CarrierFrameHeader, CarrierFrameHeaderV1},
        FrameGenerator, FrameParser,
    },
    types::{Address, Ident32, InMemoryEnvelope, Recipient, SequenceIdV1},
};
use std::io::{Read, Write};

/// Encode a frame for basic wire formats
pub fn encode_frame<T: Write>(stream: &mut T, f: &InMemoryEnvelope) -> Result<(), std::io::Error> {
    let mut header_buf = vec![];
    f.header.generate(&mut header_buf)?;

    let header_len: [u8; 4] = (header_buf.len() as u32).to_be_bytes();
    stream.write_all(&header_len)?;

    stream.write_all(&header_buf)?;
    stream.write_all(&f.buffer)?;

    // Happy camper
    Ok(())
}

fn read_exactly(size: usize, reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

// TODO: add better errors?
pub fn decode_frame<T: Read>(stream: &mut T) -> Result<InMemoryEnvelope, std::io::Error> {
    let header_len = u32::from_be_bytes(read_exactly(4, stream)?.try_into().unwrap());

    let header_buf = read_exactly(header_len as usize, stream)?;
    let (_, header) = CarrierFrameHeader::parse(&header_buf).unwrap();
    let header = header.unwrap();

    let mut payload_buf = read_exactly(header.get_payload_length(), stream)?;

    Ok(InMemoryEnvelope {
        header,
        buffer: payload_buf,
    })
}
