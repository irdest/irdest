use crate::{
    frame::{
        micro::parse::{cstring, maybe},
        parse::{self, maybe_cstring, maybe_id, take_id},
        FrameGenerator, FrameParser,
    },
    types::{Address, Ident32},
    MicroframeError, RatmanError, Result,
};
use nom::IResult;
use std::ffi::CString;

pub struct AddrCreate {
    pub name: Option<CString>,
    pub namespace_data: Option<Ident32>,
    // pub auto_up: bool,
}

impl FrameGenerator for AddrCreate {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self.name {
            Some(n) => buf.extend_from_slice(n.as_bytes()),
            None => buf.push(0),
        };
        match self.namespace_data {
            Some(sd) => buf.extend_from_slice(sd.as_bytes()),
            None => buf.push(0),
        };
        Ok(())
    }
}

impl FrameParser for AddrCreate {
    type Output = Self;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, maybe_name) = maybe_cstring(input).unwrap();

        let name: Option<CString> = match maybe_name {
            Ok(Some(name)) => Some(name),
            _ => None,
        };

        let (input, namespace_data) = maybe_id(input)?;

        Ok((
            input,
            Self {
                name,
                namespace_data,
            },
        ))
    }
}

pub struct AddrDestroy {
    pub addr: Address,
    pub force: bool,
}

impl FrameGenerator for AddrDestroy {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.addr.generate(buf)?;
        match self.force {
            true => buf.push(1),
            false => buf.push(0),
        }
        Ok(())
    }
}

impl FrameParser for AddrDestroy {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;
        let (input, _) = maybe(parse::take(1 as u8), input)?;

        let res = match addr {
            Some(addr) => Ok(Self { addr, force: false }),
            None => Err(MicroframeError::MissingFields(&["addr"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}

pub struct AddrUp {
    pub addr: Address,
}

impl FrameGenerator for AddrUp {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.addr.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for AddrUp {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;

        let res = match addr {
            Some(addr) => Ok(Self { addr }),
            None => Err(MicroframeError::MissingFields(&["addr"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}

pub struct AddrDown {
    pub addr: Address,
}

impl FrameGenerator for AddrDown {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.addr.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for AddrDown {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, addr) = maybe(parse::take_address, input)?;

        let res = match addr {
            Some(addr) => Ok(Self { addr }),
            None => Err(MicroframeError::MissingFields(&["addr"])),
        }
        .map_err(|e| RatmanError::Microframe(e));

        Ok((input, res))
    }
}
