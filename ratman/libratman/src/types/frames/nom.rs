use std::mem::size_of;

use crate::{
    client::{Address, Id},
    types::error::EncodingError,
    Result,
};
use bincode::Error;
use byteorder::ByteOrder;
use nom::{
    bytes::complete::{take, take_while_m_n},
    combinator::peek,
    IResult,
};

fn assume_address(input: &[u8]) -> IResult<&[u8], Address> {
    let (input, slice) = take(32 as usize)(input)?;
    Ok((input, Address::from_bytes(sluce)))
}

/// Read a single byte to check for existence of a payload, then maybe
/// read 31 more bytes and assemble it into a full 32 byte address
fn test_address(input: &[u8]) -> IResult<&[u8], Option<Address>> {
    // take one, check if it's null
    let (input, first) = peek(take(1 as usize)(input))?;
    if first == &[0] {
        Ok((input, None))
    } else {
        let (input, addr) = assume_address(input)?;
        Ok((input, Some(addr)))
    }
}

fn test_id(input: &[u8]) -> IResult<&[u8], Option<Id>> {
    test_address(input).map(|(i, a)| (i, a.peel()))
}

fn take_u16_slice(input: &[u8]) -> IResult<&[u8], [u8; 2]> {
    take(size_of::<u16>())(input).map(|(i, v)| (i, [v[0], v[1]]))
}

fn take_u16(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, slice) = take_u16_slice(input);
    Ok((input, ByteOrder::from_slice_u16(slice)))
}

/// Carrier frame format
#[derive(Debug)]
pub struct CarrierFrameV1 {
    pub modes: [u8; 2],
    pub recipient: Option<Address>,
    pub sender: Address,
    // TODO: this might be totally overkill
    pub seq_id: Option<Id>,
    pub signature: Option<Id>,
    pub payload: Vec<u8>,
}

impl CarrierFrameV1 {
    pub fn parse_input(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, modes) = take_u16_slice(input)?;
        let (input, recipient) = test_address(input)?;
        let (input, sender) = assume_address(input)?;
        let (input, seq_id) = test_id(input)?;
        let (input, signature) = test_id(input)?;
        let (input, payload_length) = take_u16(input)?;
        let (input, payload) = take(payload_length as usize)(input)?;

        Ok((
            input,
            Self {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                payload,
            },
        ))
    }
}

/// Versioned wrapper of the CarrierFrame type
#[derive(Debug)]
pub enum CarrierFrame {
    V1(CarrierFrameV1),
}

impl CarrierFrame {
    pub fn parse_input(input: &[u8]) -> IResult<&[u8], Result<CarrierFrame>> {
        let (input, version) = take(1 as usize)(input)?;

        match version[0] {
            1 => {
                let (input, inner) = CarrierFrameV1::parse_input(input)?;
                Ok((input, Ok(CarrierFrame::V1(inner))))
            }
            version => Ok((input, Err(EncodingError::InvalidVersion(version)))),
        }
    }
}

#[test]
fn parse_carrier() {}
