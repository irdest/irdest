//! Generator utilities

use crate::{
    types::{error::EncodingError, Address, Id},
    RatmanError, Result,
};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, TimeZone, Utc};

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

#[test]
fn test_u16_to_big_endian() {
    let val: u16 = 1312;
    let slice = u16_to_big_endian(val);
    assert_eq!([5, 32], slice);
}

impl FrameGenerator for u16 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u16_to_big_endian(self);
        slice.generate(buf)
    }
}

impl FrameGenerator for [u8; 2] {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(&self);
        Ok(())
    }
}

#[test]
fn test_slice_generate() {
    let val: u16 = 1312;
    let mut buf = vec![];
    val.generate(&mut buf);
    assert_eq!(buf.as_slice(), [5, 32]);
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
        let length: u16 = self
            .len()
            .try_into()
            .map_err(|_| EncodingError::FrameTooLarge(buf.len()))?;

        length.generate(buf)?;
        buf.append(&mut self);
        Ok(())
    }
}

impl<Tz: TimeZone> FrameGenerator for DateTime<Tz> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let utc_string = self.to_rfc3339();
        buf.extend_from_slice(utc_string.as_bytes());
        Ok(())
    }
}

#[test]
fn test_datetime_generate() {
    let dt = DateTime::parse_from_rfc3339("1993-06-09T21:34:22+02:00").unwrap();
    let mut buf = vec![];
    dt.generate(&mut buf).unwrap();
    assert_eq!(buf.len(), 25);
}

#[test]
fn vector_encode_decode() {
    let v = super::random_payload(32);
    let mut buf = vec![];
    v.clone().generate(&mut buf);

    assert_eq!(buf.len(), v.len() + 2);
    assert_eq!(buf[0..=1], u16_to_big_endian(v.len() as u16));
}
