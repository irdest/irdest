use crate::client::{Address, Id};
use byteorder::{BigEndian, ByteOrder};
use core::mem::size_of;

// Re-export the most common nom combinators and make sure we use the
// same ones everewhere
use nom::combinator::peek;
pub use nom::{bytes::complete::take, IResult};

/// A utility trait that represents a parsable frame entity
///
/// This trait is a slim wrapper around the nom parsing
/// infrastructure, aka a top-level parser still needs to map nom
/// errors to RatmanErrors.
pub trait FrameParser {
    type Output;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output>;
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

/// Peek one byte to determine whether
pub fn maybe_id(input: &[u8]) -> IResult<&[u8], Option<Id>> {
    maybe_address(input).map(|(i, a)| (i, a.map(|a| a.peel())))
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
