//! Client API protocol definitions
//!
//!

mod addr;
mod contact;
mod link;
mod peer;
mod recv;
mod send;

pub use addr::*;
pub use contact::*;
pub use link::*;
pub use peer::*;
pub use recv::*;
pub use send::*;

use crate::{
    frame::{
        generate::generate_cstring,
        micro::parse::vec_of,
        parse::{self, take_cstring, take_id, take_u32},
        FrameGenerator, FrameParser,
    },
    types::Ident32,
    ClientError, EncodingError, Result,
};
use nom::{bytes::complete::take, IResult};
use std::ffi::CString;

/// Sent from the router to the client when a client connects
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handshake {
    /// Indicate to the client which version of the protocol is used
    ///
    /// A client that connects with an older version MUST print an
    /// error to the user, indicating that the tools version they are
    /// using is not compatible with the Router version.
    pub client_version: [u8; 2],
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            client_version: super::VERSION,
        }
    }
}

impl FrameGenerator for Handshake {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(1);
        buf.extend_from_slice(self.client_version.as_slice());
        Ok(())
    }
}

impl FrameParser for Handshake {
    type Output = Self;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take_byte(input).unwrap();
        assert_eq!(version, 1);
        let (input, proto_version) = parse::take(2 as usize)(input)?;

        Ok((
            input,
            Self {
                client_version: proto_version.try_into().expect("wat??"),
            },
        ))
    }
}

/// Router-client ping and response type
#[derive(Debug)]
pub enum ServerPing {
    /// A generic "everything is good" response
    Ok,
    /// Indicate that subscriptions have data available
    ///
    /// This is only the case when the subscription is currently idle, meaning
    /// the client is not actively listening to events on the given subscription
    /// socket.  Active subscriptions are not included in this set!
    Update {
        available_subscriptions: Vec<Ident32>,
    },
    /// Communicate some kind of API error to the calling client
    Error(ClientError),
    ///
    IncompatibleVersion { router: CString, client: CString },
    /// Connection timed out
    Timeout,
    /// Subscription response type
    Subscription { sub_id: Ident32, sub_bind: CString },
}

impl FrameGenerator for ServerPing {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::Ok => buf.push(1),
            Self::Update {
                available_subscriptions,
            } => {
                buf.push(2);
                available_subscriptions.generate(buf)?;
            }
            Self::Error(error) => {
                buf.push(3);
                let mut err_buf = bincode::serialize(&error)?;
                let mut len_buf = vec![];
                (buf.len() as u32).generate(&mut len_buf)?;
                buf.append(&mut len_buf);
                buf.append(&mut err_buf);
            }
            Self::Timeout => buf.push(4),
            Self::IncompatibleVersion { router, client } => {
                buf.push(5);
                generate_cstring(router, buf)?;
                generate_cstring(client, buf)?;
            }
            Self::Subscription { sub_id, sub_bind } => {
                buf.push(6);
                Some(sub_id).generate(buf)?;
                generate_cstring(sub_bind, buf)?;
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
            1 => Ok(Self::Ok),
            2 => {
                let (_input, available_subscriptions) = vec_of(take_id, input)?;
                input = _input; // wish we didn't need this weirdness
                Ok(Self::Update {
                    available_subscriptions,
                })
            }
            3 => {
                let (_input, err_len) = take_u32(input)?;
                let (_input, err_buf) = take(err_len as usize)(input)?;
                let err = bincode::deserialize(&err_buf).unwrap();
                input = _input;
                Ok(Self::Error(err))
            }
            4 => Ok(Self::Timeout),
            5 => {
                let (_input, router) = take_cstring(input)?;
                let (_input, client) = take_cstring(_input)?;
                input = _input;
                Ok(Self::IncompatibleVersion {
                    router: router.expect("failed to decode version string"),
                    client: client.expect("failed to decode version string"),
                })
            }
            6 => {
                let (_input, sub_id) = take_id(input)?;
                let (_input, sub_bind) = take_cstring(input)?;
                input = _input;
                sub_bind.map(|sub_bind| Self::Subscription { sub_id, sub_bind })
            }
            _ => Err(EncodingError::Parsing(format!("Invalid ServerPing type={}", tt)).into()),
        };

        Ok((input, output))
    }
}
