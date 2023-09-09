//! Generator utilities

use crate::{
    types::{Address, Id},
    Result,
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
        Ok(())
    }
}
