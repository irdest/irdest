use crate::{
    frame::{
        micro::parse::{cstring, maybe},
        parse, FrameGenerator, FrameParser,
    },
    types::Address,
    MicroframeError, RatmanError, Result,
};
use nom::IResult;
use std::ffi::CString;

pub struct AddrCreate {
    pub name: Option<CString>,
    // pub auto_up: bool,
}

impl FrameGenerator for AddrCreate {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self.name {
            Some(n) => buf.extend_from_slice(n.as_bytes()),
            None => buf.push(0),
        };
        Ok(())
    }
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
