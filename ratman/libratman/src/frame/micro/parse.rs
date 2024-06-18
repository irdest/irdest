use byteorder::{BigEndian, ByteOrder};
use nom::{
    bytes::complete::{take, take_till},
    combinator::peek,
    IResult,
};
use std::ffi::CString;

/// Peek a single byte to check if data is present, if so, execute the
/// child parser
pub fn maybe<'a, O, F>(mut child: F, input: &'a [u8]) -> IResult<&'a [u8], Option<O>>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
{
    let (input, first) = peek(take(1 as usize))(input)?;
    if first == &[0] {
        // Take the byte we just peeked to burn it
        let (input, _) = take(1 as usize)(input)?;
        Ok((input, None))
    } else {
        let (input, result) = child(input)?;
        Ok((input, Some(result)))
    }
}

/// Read a single byte as to how many elements will follow.  Then call
/// the child parser for every element.  Each element must be the
/// same, but can be varying sizes, according to the way they can be
/// incrementally parsed!
///
/// With a single byte as length only 255 items per vector encoding
/// are supported at the moment!
pub fn vec_of<'a, O, F>(mut child: F, input: &'a [u8]) -> IResult<&'a [u8], Vec<O>>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
{
    let (mut input, count) = take(2 as usize)(input)?;
    let count = u16::from_be_bytes([count[0], count[1]]);
    eprintln!("Reading {count} length vector");

    let mut buf = Vec::with_capacity(count as usize);
    for _ in 0..count as usize {
        let (new_input, item) = child(input)?;
        input = new_input;
        buf.push(item);
    }

    Ok((input, buf))
}

/// Attempt to parse a CString from the input
pub fn cstring(input: &[u8]) -> IResult<&[u8], Option<CString>> {
    let (input, bytes) = take_till(|x| x == '\0' as u8)(input)?;
    Ok((input, CString::from_vec_with_nul(bytes.to_vec()).ok()))
}
