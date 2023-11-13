use crate::{
    client::{Address, Id},
    types::{error::EncodingError, frames::CarrierFrameHeader, Recipient, SequenceIdV1},
    RatmanError, Result,
};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use core::mem::size_of;
use std::ffi::CString;

// Re-export the most common nom combinators and make sure we use the
// same ones everewhere
pub use nom::{bytes::complete::take, IResult};
use nom::{
    bytes::complete::{take_till, take_while1},
    combinator::{opt, peek},
    Parser,
};

/// A utility trait that represents a parsable frame entity
///
/// This trait is a slim wrapper around the nom parsing
/// infrastructure, aka a top-level parser still needs to map nom
/// errors to RatmanErrors.
pub trait FrameParser {
    type Output;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output>;
}

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

pub fn take_address(input: &[u8]) -> IResult<&[u8], Address> {
    let (input, slice) = take(32 as usize)(input)?;
    Ok((input, Address::from_bytes(slice)))
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
pub fn maybe_id(input: &[u8]) -> IResult<&[u8], Option<Id>> {
    maybe_address(input).map(|(i, a)| (i, a.map(|a| a.peel())))
}

/// Take 32 bytes for an Id
pub fn take_id(input: &[u8]) -> IResult<&[u8], Id> {
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
