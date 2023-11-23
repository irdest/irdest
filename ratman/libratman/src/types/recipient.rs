use crate::{
    frame::{parse::take_address, FrameGenerator, FrameParser},
    types::Address,
    Result,
};
use nom::{bytes::complete::take, IResult};
use serde::{Deserialize, Serialize};

/// Represent a message recipient on the network
///
/// This can either be a single address, or an address namespace for
/// flooding.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Recipient {
    /// Contains a single targeted message
    Address(Address),
    /// Contains a flood namespace
    Namespace(Address),
}

impl Recipient {
    pub fn inner_address(&self) -> Address {
        match self {
            Self::Address(addr) => *addr,
            Self::Namespace(addr) => *addr,
        }
    }
}

impl FrameGenerator for Option<Recipient> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(Recipient::Address(addr)) => {
                buf.push(1);
                addr.generate(buf)?;
            }
            Some(Recipient::Namespace(addr)) => {
                buf.push(2);
                addr.generate(buf)?;
            }
            None => {
                buf.push(0);
            }
        }

        Ok(())
    }
}

impl FrameParser for Option<Recipient> {
    type Output = Self;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, mode) = take(1 as usize)(input)?;

        match mode[0] {
            0 => Ok((input, None)),
            1 => {
                let (input, addr) = take_address(input)?;
                Ok((input, Some(Recipient::Address(addr))))
            }
            2 => {
                let (input, addr) = take_address(input)?;
                Ok((input, Some(Recipient::Namespace(addr))))
            }
            _ => {
                unreachable!(
                    "this is definitely reachable but you've
            been naughty with your data"
                )
            }
        }
    }
}
