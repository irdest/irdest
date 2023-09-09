//! Generator utilities

use crate::{
    types::{error::EncodingError, Address, Id},
    RatmanError, Result,
};
use byteorder::{BigEndian, ByteOrder};

/// A utility trait that represents a serialisable frame entity
///
/// This trait should be implemented for frame sub-types to avoid code
/// duplication when serialising entities.  Additionally this trait
/// consumes the given frame to avoid duplication.
pub trait FrameGenerator {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()>;
}

fn u16_to_big_endian(val: u16) -> [u8; 2] {
    let mut v = [0; 2];
    BigEndian::write_u16(&mut v, val);
    v
}

impl FrameGenerator for u16 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u16_to_big_endian(self);
        slice.generate(buf)
    }
}

impl FrameGenerator for [u8; 2] {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self[0]);
        buf.push(self[1]);
        Ok(())
    }
}

impl FrameGenerator for Id {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

impl FrameGenerator for Option<Id> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // If the Id is None we simply push a zero-byte
        match self {
            Some(id) => buf.extend_from_slice(id.as_bytes()),
            None => buf.push(0),
        }

        Ok(())
    }
}

impl FrameGenerator for Address {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

impl FrameGenerator for Option<Address> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // If the Address is None we simply push a zero-byte
        match self {
            Some(id) => buf.extend_from_slice(id.as_bytes()),
            None => buf.push(0),
        }

        Ok(())
    }
}

impl FrameGenerator for Vec<u8> {
    fn generate(mut self, buf: &mut Vec<u8>) -> Result<()> {
        // First we push the length of the vector as a u16, then the vector itself
        let length: u16 = buf
            .len()
            .try_into()
            .map_err(|_| EncodingError::FrameTooLarge(buf.len()))?;

        length.generate(buf)?;
        buf.append(&mut self);
        Ok(())
    }
}
