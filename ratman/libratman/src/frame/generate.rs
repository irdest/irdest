//! Generator utilities

use crate::{frame::FrameGenerator, types::Ident32, EncodingError, Result};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, TimeZone};
use std::{ffi::CString, time::Duration};

fn u16_to_big_endian(val: u16) -> [u8; 2] {
    let mut v = [0; 2];
    BigEndian::write_u16(&mut v, val);
    v
}

fn u32_to_big_endian(val: u32) -> [u8; 4] {
    let mut v = [0; 4];
    BigEndian::write_u32(&mut v, val);
    v
}

fn u64_to_big_endian(val: u64) -> [u8; 8] {
    let mut v = [0; 8];
    BigEndian::write_u64(&mut v, val);
    v
}

fn u128_to_big_endian(val: u128) -> [u8; 16] {
    let mut v = [0; 16];
    BigEndian::write_u128(&mut v, val);
    v
}

#[test]
fn test_u16_to_big_endian() {
    let val: u16 = 1312;
    let slice = u16_to_big_endian(val);
    assert_eq!([5, 32], slice);
}

impl FrameGenerator for u8 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self);
        Ok(())
    }
}

impl FrameGenerator for u16 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u16_to_big_endian(self);
        slice.generate(buf)
    }
}

impl FrameGenerator for u32 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u32_to_big_endian(self);
        slice.generate(buf)
    }
}

impl FrameGenerator for Option<u32> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(s) => {
                buf.push(1);
                s.generate(buf)?;
                Ok(())
            }
            None => {
                buf.push(0);
                Ok(())
            }
        }
    }
}

impl FrameGenerator for u64 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u64_to_big_endian(self);
        slice.generate(buf)
    }
}

impl FrameGenerator for Option<u64> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(s) => {
                buf.push(1);
                s.generate(buf)?;
                Ok(())
            }
            None => {
                buf.push(0);
                Ok(())
            }
        }
    }
}

impl FrameGenerator for u128 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let slice = u128_to_big_endian(self);
        slice.generate(buf)
    }
}

#[test]
fn test_slice_generate() {
    let val: u16 = 1312;
    let mut buf = vec![];
    let _ = val.generate(&mut buf);
    assert_eq!(buf.as_slice(), [5, 32]);
}

impl FrameGenerator for Ident32 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

impl FrameGenerator for Option<Ident32> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // If the Id is None we simply push a zero-byte
        match self {
            Some(id) => buf.extend_from_slice(id.as_bytes()),
            None => buf.push(0),
        }

        Ok(())
    }
}

// Implement FrameGenerator for any array
impl<const LENGTH: usize> FrameGenerator for [u8; LENGTH] {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(&self);
        Ok(())
    }
}

impl<const LENGTH: usize> FrameGenerator for Option<[u8; LENGTH]> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            // If the signature exists, we write it
            Some(ref this) => buf.extend_from_slice(this),
            // Otherwise write a zero-byte
            None => buf.push(0),
        }
        Ok(())
    }
}

// impl FrameGenerator for Vec<u8> {
//     fn generate(mut self, buf: &mut Vec<u8>) -> Result<()> {
//         // First we push the length of the vector as a u16, then the vector itself
//         let length: u16 = self
//             .len()
//             .try_into()
//             .map_err(|_| EncodingError::FrameTooLarge(buf.len()))?;

//         length.generate(buf)?;
//         buf.append(&mut self);
//         Ok(())
//     }
// }

impl<T: FrameGenerator> FrameGenerator for Vec<T> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // First we push the length of the vector as a u16, then the vector itself
        let length: u16 = self
            .len()
            .try_into()
            .map_err(|_| EncodingError::FrameTooLarge(buf.len()))?;
        length.generate(buf)?;

        for item in self {
            item.generate(buf)?;
        }

        Ok(())
    }
}

impl<Tz: TimeZone> FrameGenerator for DateTime<Tz> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        let utc_string = self.to_rfc3339();
        // todo: replace this with a better encoding?
        buf.extend_from_slice(utc_string.as_bytes());
        Ok(())
    }
}

pub fn generate_cstring(cstr: CString, buf: &mut Vec<u8>) -> Result<()> {
    buf.extend_from_slice(cstr.as_bytes_with_nul());
    Ok(())
}

pub fn generate_option_cstring(opt: Option<CString>, buf: &mut Vec<u8>) -> Result<()> {
    match opt {
        Some(cstr) => generate_cstring(cstr, buf)?,
        None => buf.push(0),
    }
    Ok(())
}

pub fn generate_cstring_tuple(tup: (CString, CString), buf: &mut Vec<u8>) -> Result<()> {
    generate_cstring(tup.0, buf)?;
    generate_cstring(tup.1, buf)?;
    Ok(())
}

pub fn generate_cstring_tuple_vec(vec: Vec<(CString, CString)>, buf: &mut Vec<u8>) -> Result<()> {
    // first write out the length of the vector as a u16
    assert!(vec.len() < core::u16::MAX as usize);
    let len = vec.len() as u16;
    len.generate(buf)?;

    // then we iterate through the vector and write that many items
    vec.into_iter()
        .map(|tup| generate_cstring_tuple(tup, buf))
        .fold(Ok(()), |acc, ret| match (acc, ret) {
            (Ok(()), Err(e)) => Err(e),
            (e, _) => e,
        })?;
    Ok(())
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
    let v = crate::frame::carrier::random_payload(32);
    let mut buf = vec![];
    let _ = v.clone().generate(&mut buf);

    assert_eq!(buf.len(), v.len() + 2);
    assert_eq!(buf[0..=1], u16_to_big_endian(v.len() as u16));
}

/// Read any `repr(C)` type as binary data for serialisation
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of_val(p))
}
