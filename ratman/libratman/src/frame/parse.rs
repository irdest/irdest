use crate::{
    types::{Address, Ident32},
    EncodingError, Result,
};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use core::mem::size_of;
use std::ffi::CString;

// Re-export the most common nom combinators and make sure we use the
// same ones everewhere
pub use nom::{bytes::complete::take, IResult};
use nom::{bytes::complete::take_while1, combinator::peek};

use super::FrameParser;

/// Peek one byte to check if the next section exists, if so, read LEN
/// bytes, otherwise burn the zero byte
pub fn maybe<const LEN: usize>(input: &[u8]) -> IResult<&[u8], Option<[u8; LEN]>> {
    // take one, check if it's null
    let (input, first) = peek(take(1 as usize))(input)?;
    if first == &[0] {
        // Take the byte we just peeked into to burn it
        let (input, _) = take(1 as usize)(input)?;
        Ok((input, None))
    } else {
        let (input, maybe_slice) = take(LEN)(input).map(|(i, s)| {
            let mut buf = [0; LEN];
            buf.copy_from_slice(&s);
            (i, Some(buf))
        })?;

        Ok((input, maybe_slice))
    }
}

pub fn take_u32(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, slice) = take(core::mem::size_of::<u32>())(input)?;
    Ok((input, BigEndian::read_u32(slice)))
}

pub fn take_u64(input: &[u8]) -> IResult<&[u8], u64> {
    let (input, slice) = take(core::mem::size_of::<u64>())(input)?;
    Ok((input, BigEndian::read_u64(slice)))
}

pub fn take_address(input: &[u8]) -> IResult<&[u8], Address> {
    let (input, slice) = take(32 as usize)(input)?;
    Ok((input, Address::from_bytes(slice)))
}

pub fn take_cstring(input: &[u8]) -> IResult<&[u8], Result<CString>> {
    let (input, bytes) = take_while1(|c| c as char != '\0')(input)?;
    Ok((
        input,
        CString::new(bytes).map_err(|c| EncodingError::Parsing(format!("{:?}", c)).into()),
    ))
}

pub fn maybe_cstring(input: &[u8]) -> IResult<&[u8], Result<Option<CString>>> {
    let (input, first) = peek(take_byte)(input)?;
    if first != 0 {
        let (input, cstr) = take_cstring(input)?;
        Ok((input, cstr.map(|c| Some(c))))
    } else {
        let (input, _) = take(1 as usize)(input)?;
        Ok((input, Ok(None)))
    }
}

pub fn take_cstring_vec(input: &[u8]) -> IResult<&[u8], Result<Vec<CString>>> {
    let (mut input, num) = take_u16(input)?;

    let mut cstr;
    let mut buf = vec![];
    for i in 0..num {
        (input, cstr) = maybe_cstring(input)?;
        match cstr {
            Ok(Some(c)) => buf.push(c),
            Ok(None) => {
                return Ok((
                    input,
                    Err(EncodingError::Parsing(format!(
                        "Tried to read {} strings, but only {} were provided",
                        num, i
                    ))
                    .into()),
                ));
            }
            Err(e) => return Ok((input, Err(e.into()))),
        }
    }

    Ok((input, Ok(buf)))
}

pub fn take_cstring_tuple_vec(input: &[u8]) -> IResult<&[u8], Result<Vec<(CString, CString)>>> {
    let (mut input, num) = take_u16(input)?;

    let (mut cstr_key, mut cstr_val);
    let mut buf = vec![];
    for i in 0..num {
        (input, cstr_key) = maybe_cstring(input)?;
        (input, cstr_val) = maybe_cstring(input)?;
        match (cstr_key, cstr_val) {
            (Ok(Some(c)), Ok(Some(c2))) => buf.push((c, c2)),
            (Ok(Some(_)), Ok(None)) | (Ok(None), Ok(Some(_))) | (Ok(None), Ok(None)) => {
                return Ok((
                    input,
                    Err(EncodingError::Parsing(format!(
                        "Tried to read {} string tuples, but only {} were provided",
                        num, i
                    ))
                    .into()),
                ));
            }
            (Err(e), _) | (_, Err(e)) => return Ok((input, Err(e.into()))),
        }
    }

    Ok((input, Ok(buf)))
}

/// Read a single byte to check for existence of a payload, then maybe
/// read 31 more bytes and assemble it into a full 32 byte address
pub fn maybe_address(input: &[u8]) -> IResult<&[u8], Option<Address>> {
    // take one, check if it's null
    let (input, first) = peek(take(1 as usize))(input)?;
    if first == &[0] {
        // Take the byte we just peeked into to burn it
        let (input, _) = take(1 as usize)(input)?;
        Ok((input, None))
    } else {
        let (input, addr) = take_address(input)?;
        Ok((input, Some(addr)))
    }
}

/// Take a single byte
pub fn take_byte(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, byte) = take(1 as usize)(input)?;
    Ok((input, byte[0]))
}

/// Peek one byte and then maybe take an Id
pub fn maybe_id(input: &[u8]) -> IResult<&[u8], Option<Ident32>> {
    maybe_address(input).map(|(i, a)| (i, a.map(|a| a.peel())))
}

/// Take 32 bytes for an Id
pub fn take_id(input: &[u8]) -> IResult<&[u8], Ident32> {
    take_address(input).map(|(i, a)| (i, a.peel()))
}

/// Take two bytes and return them as a raw u16 slice
pub fn take_u16_slice(input: &[u8]) -> IResult<&[u8], [u8; 2]> {
    take(size_of::<u16>())(input).map(|(i, v)| (i, [v[0], v[1]]))
}

/// Take two bytes and read them as a bigendian u16
pub fn take_u16(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, slice) = take_u16_slice(input)?;
    Ok((input, BigEndian::read_u16(&slice)))
}

pub fn take_datetime(input: &[u8]) -> IResult<&[u8], Result<DateTime<Utc>>> {
    // Take 25 bytes which is the length of an rfc3339 timestamp
    let (input, slice) = take(25 as usize)(input)?;

    // Convert this to a string and fail early if it's an invalid string encoding
    let dt_str: Result<_> = core::str::from_utf8(slice)
        .map_err(|e| EncodingError::Parsing(format!("invalid datetime encoding: {}", e)).into());

    Ok((
        input,
        // If it was a string, try to parse an rfc3339 timestamp
        dt_str.and_then(|dt_str| {
            // And turn it into a Utc timestamp
            DateTime::parse_from_rfc3339(dt_str)
                .map(|tz_offset| tz_offset.into())
                .map_err(|e| EncodingError::Parsing(format!("invalid timestamp: {}", e)).into())
        }),
    ))
}

pub fn maybe_signature(input: &[u8]) -> IResult<&[u8], Option<[u8; 64]>> {
    let (input, first) = peek(take(1 as usize))(input)?;
    if first == &[0] {
        let (input, slice) = take(64 as usize)(input)?;
        let mut signature = [0; 64];
        signature.copy_from_slice(slice);
        Ok((input, Some(signature)))
    } else {
        let (input, _) = take(1 as usize)(input)?;
        Ok((input, None))
    }
}

impl<T: FrameParser> FrameParser for Vec<T> {
    type Output = Vec<T::Output>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (mut input, length) = take_u16(input)?;

        let mut buf = vec![];

        for _ in 0..length {
            let (_input, item) = T::parse(input)?;
            buf.push(item);
            input = _input;
        }

        Ok((input, buf))
    }
}

////// Test that CStrings can be encoded correctly

#[test]
fn test_cstring() {
    use crate::frame::generate::generate_cstring;
    let data = "Hello to the world!";
    let c = CString::new(data.as_bytes()).unwrap();

    let mut cbuf = vec![];
    generate_cstring(c, &mut cbuf).unwrap();
    assert_eq!(String::from_utf8(cbuf).unwrap(), format!("{}\0", data));
}

// #[test]
// fn test_cstring_vec() {
//     use crate::frame::generate::generate_cstring;
//     let data = "Hello to the world!";
//     let c = CString::new(data.as_bytes()).unwrap();

//     let mut cbuf = vec![];
//     generate_cstring(c, &mut cbuf).unwrap();

//     assert_eq!(cbuf.as_bytes(), format!("{}\0", data).as_bytes());
// }
