use crate::{
    frame::carrier::{parse, FrameParser},
    frame::micro::parse::{cstring, maybe},
    types::{Address, Id},
    MicroframeError, RatmanError, Result,
};
use chrono::format::parse;
use nom::{branch::alt, IResult, Parser};
use std::ffi::CString;

#[repr(C)]
pub struct AddrCreate {
    name: Option<CString>,
}

impl FrameParser for AddrCreate {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, res) = match maybe(cstring, input) {
            Ok((input, Some(maybe_valid))) => Ok((
                input,
                maybe_valid.map_or_else(
                    || Err(RatmanError::Microframe(MicroframeError::InvalidString)),
                    |name| Ok(Self { name: Some(name) }),
                ),
            ))?,
            Ok((input, None)) => Ok((input, Ok(Self { name: None })))?,
            Err(e) => Err(e)?,
        };

        Ok((input, res))
    }
}

#[repr(C)]
pub struct AddrDestroy {
    addr: Address,
    token: Id,
}

impl FrameParser for AddrDestroy {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;
        let (input, token) = maybe(parse::take_id, input)?;

        let res = match (addr, token) {
            (Some(addr), Some(token)) => Ok(Self { addr, token }),
            (None, Some(_)) => Err(MicroframeError::MissingFields(&["addr"])),
            (Some(_), None) => Err(MicroframeError::MissingFields(&["token"])),
            (None, None) => Err(MicroframeError::MissingFields(&["addr", "token"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}

#[repr(C)]
pub struct AddrUp {
    addr: Address,
    token: Id,
}

impl FrameParser for AddrUp {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;
        let (input, token) = maybe(parse::take_id, input)?;

        let res = match (addr, token) {
            (Some(addr), Some(token)) => Ok(Self { addr, token }),
            (None, Some(_)) => Err(MicroframeError::MissingFields(&["addr"])),
            (Some(_), None) => Err(MicroframeError::MissingFields(&["token"])),
            (None, None) => Err(MicroframeError::MissingFields(&["addr", "token"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}

#[repr(C)]
pub struct AddrDown {
    addr: Address,
    token: Id,
}

impl FrameParser for AddrDown {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;
        let (input, token) = maybe(parse::take_id, input)?;

        let res = match (addr, token) {
            (Some(addr), Some(token)) => Ok(Self { addr, token }),
            (None, Some(_)) => Err(MicroframeError::MissingFields(&["addr"])),
            (Some(_), None) => Err(MicroframeError::MissingFields(&["token"])),
            (None, None) => Err(MicroframeError::MissingFields(&["addr", "token"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}

#[repr(C)]
pub enum AddrCommand {
    Create(AddrCreate),
    Destroy(AddrDestroy),
    Up(AddrUp),
    Down(AddrDown),
}

impl FrameParser for AddrCommand {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        alt((
            AddrCreate::parse.map(|x| x.map(|inner| Self::Create(inner))),
            AddrDestroy::parse.map(|x| x.map(|inner| Self::Destroy(inner))),
            AddrUp::parse.map(|x| x.map(|inner| Self::Up(inner))),
            AddrDown::parse.map(|x| x.map(|inner| Self::Down(inner))),
        ))(input)
    }
}
