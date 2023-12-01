//! Client API protocol definitions
//!
//!

mod addr;
mod contact;
mod link;
mod peer;
mod recv;

pub use addr::*;
pub use contact::*;
pub use link::*;
use nom::IResult;
pub use peer::*;
pub use recv::*;

use crate::{
    frame::{
        micro::parse::vec_of,
        parse::{self, take_id},
        FrameGenerator, FrameParser,
    },
    types::Id,
    Result,
};

/// Sent from the router to the client when a client connects
pub struct Handshake {
    /// Indicate to the client which version of the protocol is used
    ///
    /// A client that connects with an older version MUST print an
    /// error to the user, indicating that the tools version they are
    /// using is not compatible with the Router version.
    pub proto_version: [u8; 2],
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            proto_version: super::VERSION,
        }
    }
}

impl FrameGenerator for Handshake {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(1);
        buf.extend_from_slice(self.proto_version.as_slice());
        Ok(())
    }
}

impl FrameParser for Handshake {
    type Output = Self;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take_byte(input)?;
        assert_eq!(version, 1);
        let (input, proto_version) = parse::take(2 as usize)(input)?;

        Ok((
            input,
            Self {
                proto_version: proto_version.try_into().expect("wat??"),
            },
        ))
    }
}

/// Sent from the router to the client on every 'ping'
pub struct Ping {
    /// Indicate to the client which subscription IDs are available
    ///
    /// A client can then decide to pull a particular subscription Id
    /// to get the next message stream for that subscription
    pub available_subscriptions: Vec<Id>,
}

impl FrameGenerator for Ping {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(1);
        self.available_subscriptions.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for Ping {
    type Output = Self;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take_byte(input)?;
        assert_eq!(version, 1);
        let (input, available_subscriptions) = vec_of(take_id, input)?;

        Ok((
            input,
            Self {
                available_subscriptions,
            },
        ))
    }
}
