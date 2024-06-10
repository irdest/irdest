//! Client API protocol definitions
//!
//!

mod addr;
mod contact;
mod link;
mod peer;
mod recv;

use std::ffi::CString;

pub use addr::*;
pub use contact::*;
pub use link::*;
use nom::IResult;
pub use peer::*;
pub use recv::*;

use crate::{
    frame::{
        generate::generate_cstring,
        micro::parse::vec_of,
        parse::{self, take_cstring, take_id},
        FrameGenerator, FrameParser,
    },
    types::Id,
    EncodingError, RatmanError, Result,
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
pub enum ServerPing {
    /// Indicate to the client which subscription IDs are available
    ///
    /// A client can then decide to pull a particular subscription Id
    /// to get the next message stream for that subscription
    Update { available_subscriptions: Vec<Id> },
    /// Communicate some kind of API error to the calling client
    Error(CString),
    /// Connection timed out
    Timeout,
}

impl FrameGenerator for ServerPing {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::Update {
                available_subscriptions,
            } => {
                buf.push(1);
                available_subscriptions.generate(buf)?;
            }
            Self::Error(error) => {
                buf.push(2);
                generate_cstring(error, buf)?;
            }
            Self::Timeout => {
                buf.push(3);
            }
        }

        Ok(())
    }
}

impl FrameParser for ServerPing {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (mut input, tt) = parse::take_byte(input)?;

        let output = match tt {
            1 => {
                let (_input, available_subscriptions) = vec_of(take_id, input)?;
                input = _input; // wish we didn't need this weirdness
                Ok(Self::Update {
                    available_subscriptions,
                })
            }
            2 => {
                let (_input, err_str) = take_cstring(input)?;
                input = _input;
                Ok(Self::Error(err_str.expect("failed to decode error string")))
            }
            3 => Ok(Self::Timeout),
            _ => Err(EncodingError::Parsing(format!("Invalid ServerPing type={}", tt)).into()),
        };

        Ok((input, output))
    }
}
