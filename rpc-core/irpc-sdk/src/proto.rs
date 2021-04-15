//! Basic protocol definitions for Service <-> Broker commands

use crate::{
    error::{RpcError, RpcResult},
    io::{self, Message},
    Capabilities, Identity,
};
use serde::Serialize;

/// A message registering a service with the broker
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    /// Service name
    pub name: String,
    /// Service version
    pub version: u16,
    /// Service human-friendly description
    pub description: String,
    /// Capability set
    pub caps: Capabilities,
}

impl Registry {
    pub fn parse(vec: &Message) -> RpcResult<Registry> {
        Ok(serde_json::from_str(
            std::str::from_utf8(&vec.data).unwrap(),
        )?)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SdkCommand {
    Shutdown {
        name: String,
        hash_id: Identity,
    },
    Upgrade {
        name: String,
        hash_id: Identity,
        version: u16,
    },
    Subscription(SubscriptionCmd),
}

impl SdkCommand {
    pub fn parse(enc: u8, msg: &Message) -> RpcResult<Self> {
        io::decode(enc, &msg.data)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SdkReply {
    /// The operation was successful
    Ok,
    /// Rturn an identity
    Identity(Identity),
    /// An error occured
    Error(RpcError),
}

impl From<RpcResult<()>> for SdkReply {
    fn from(r: RpcResult<()>) -> Self {
        match r {
            Ok(()) => Self::Ok,
            Err(e) => Self::Error(e),
        }
    }
}

impl From<RpcResult<Identity>> for SdkReply {
    fn from(r: RpcResult<Identity>) -> Self {
        match r {
            Ok(id) => Self::Identity(id),
            Err(e) => Self::Error(e),
        }
    }
}


impl SdkReply {
    pub fn parse_identity(enc: u8, msg: &Message) -> RpcResult<Identity> {
        match io::decode(enc, &msg.data)? {
            SdkReply::Identity(id) => Ok(id),
            SdkReply::Error(e) => Err(e),
            _ => Err(RpcError::UnexpectedPayload),
        }
    }

    pub fn parse_ok(enc: u8, msg: &Message) -> RpcResult<()> {
        match io::decode(enc, &msg.data)? {
            SdkReply::Ok => Ok(()),
            SdkReply::Error(e) => Err(e),
            _ => Err(RpcError::UnexpectedPayload),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SubscriptionCmd {
    /// Register a subscription
    Register(Vec<u8>),
    /// Unregister a subscription
    Unregister(Identity),
    /// A subscription push event
    Push(Vec<u8>),
}

impl SubscriptionCmd {
    /// Create a subscription register message
    pub fn register<T: Serialize>(enc: u8, t: T) -> RpcResult<Self> {
        Ok(Self::Register(io::encode(enc, &t)?))
    }

    /// Create a subscription unregister message
    pub fn unregister(id: Identity) -> Self {
        Self::Unregister(id)
    }

    /// Create a subscription push message
    pub fn push<T: Serialize>(enc: u8, t: T) -> RpcResult<Self> {
        Ok(Self::Push(io::encode(enc, &t)?))
    }
}

/// This test is as much a test of how the message structures compose
/// as well as how to send a registry message to the broker
#[test]
fn registry_encode_decode() {
    use crate::{io, ENCODING_JSON};

    let reg = Registry {
        name: "org.irdest.test".into(),
        version: 2,
        description: "A simple test service".into(),
        caps: Capabilities::basic_json(),
    };

    // Encode the registry message as json and then create a message
    let data = io::encode(ENCODING_JSON, &reg).unwrap();
    let msg = Message::to_addr("org.irdest._broker", "ord.irdest.test", data);

    // Encode the message wrapper as json
    let msg_data = io::encode(ENCODING_JSON, &msg).unwrap();

    /////// Let's pretend we're the message broker now

    // First turn the binary stuff into a Message wrapper
    let msg2: Message = io::decode(ENCODING_JSON, &msg_data).unwrap();

    // Then try to parse the registry message
    let reg2 = Registry::parse(&msg2).unwrap();

    assert_eq!(reg, reg2);
}
