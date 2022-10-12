mod lookup;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use protobuf::Message as _;
use ratman_client::{Address, Id, RatmanIpc, TimePair};
use std::convert::TryFrom;

pub use lookup::Lookup;

pub const NAMESPACE: [u8; 32] = [
    0xF3, 0xFA, 0x1B, 0xCC, 0x57, 0x01, 0x7A, 0xCF, 0x57, 0x4C, 0x0F, 0xCF, 0x2E, 0x6F, 0x4F, 0x2B,
    0x24, 0x02, 0x90, 0x36, 0xE0, 0x0D, 0xC9, 0x25, 0xFA, 0xCC, 0xBB, 0x53, 0x5F, 0x80, 0x5E, 0x48,
];

#[cfg(feature = "proto")]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto_gen/mod.rs"));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub header: Header,
    pub payload: Vec<u8>,
}

impl Envelope {
    pub fn message(msg: &ratman_client::Message) -> Self {
        Self {
            header: Header::message(&msg),
            payload: msg.get_payload(),
        }
    }

    pub fn into_proto(self) -> proto::db::Envelope {
        proto::db::Envelope {
            id: self.header.id.as_bytes().into(),
            time_ms: self.header.time.timestamp_millis(),
            sender: self
                .header
                .sender
                .map(|v| v.as_bytes().into())
                .unwrap_or_default(),
            payload: self.payload,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub id: Id,
    pub time: DateTime<Utc>,
    pub sender: Option<Address>,
}

impl Header {
    pub fn message(msg: &ratman_client::Message) -> Self {
        Self {
            id: msg.get_id(),
            time: msg.get_time().local(),
            sender: Some(msg.get_sender()),
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            id: Id::random(),
            time: TimePair::sending().local(),
            sender: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub header: Header,
    pub payload: Payload,
}

impl Message {
    pub fn new<T: Into<Payload>>(p: T) -> Self {
        Self {
            header: Header::default(),
            payload: p.into(),
        }
    }

    pub fn into_proto(self) -> proto::feed::Message {
        proto::feed::Message {
            payload: Some(self.payload.into()),
            ..Default::default()
        }
    }
}

impl TryFrom<&ratman_client::Message> for Message {
    type Error = anyhow::Error;

    fn try_from(msg: &ratman_client::Message) -> Result<Self, Self::Error> {
        let p = proto::feed::Message::parse_from_bytes(&msg.get_payload()[..])?;
        Ok(Self {
            header: Header::message(msg),
            payload: p.payload.ok_or(anyhow!("message has no payload?"))?.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Post(Post),
}

impl From<Post> for Payload {
    fn from(p: Post) -> Self {
        Self::Post(p)
    }
}

impl From<proto::feed::Message_oneof_payload> for Payload {
    fn from(v: proto::feed::Message_oneof_payload) -> Self {
        use proto::feed::Message_oneof_payload;
        match v {
            Message_oneof_payload::post(p) => Self::Post(p.into()),
        }
    }
}

impl Into<proto::feed::Message_oneof_payload> for Payload {
    fn into(self) -> proto::feed::Message_oneof_payload {
        use proto::feed::Message_oneof_payload;
        match self {
            Self::Post(p) => Message_oneof_payload::post(p.into()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub nick: String,
    pub text: String,
    pub topic: String,
}

impl From<proto::feed::Post> for Post {
    fn from(v: proto::feed::Post) -> Self {
        Self {
            nick: v.nick,
            text: v.text,
            topic: v.topic,
        }
    }
}

impl Into<proto::feed::Post> for Post {
    fn into(self) -> proto::feed::Post {
        let mut p = proto::feed::Post::new();
        p.set_nick(self.nick);
        p.set_text(self.text);
        p.set_topic(self.topic);
        p
    }
}

/// Loads an address from a file ('addr' in the system-appropriate config dir), or
/// if that doesn't exist, call the local ratmand to generate one, stashing it in
/// said file to be found on our next run.
pub async fn load_or_create_addr() -> Result<Address> {
    // Find our configuration directory. Make sure to respect $XDG_CONFIG_HOME!
    let dirs = directories::ProjectDirs::from("org", "irdest", "irdest-mblog")
        .ok_or(anyhow!("couldn't find config dir"))?;
    let cfg_dir = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|path| path.into())
        .unwrap_or_else(|| dirs.config_dir().to_path_buf());

    // Try to read an existing "addr" file...
    let addr_path = cfg_dir.join("addr");
    match async_std::fs::read_to_string(&addr_path).await {
        // We've done this before - use the existing address.
        Ok(s) => Ok(Address::from_string(&s)),

        // There's no "addr" file - let's create one.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Create the config directory.
            match async_std::fs::create_dir_all(&cfg_dir).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(e) => Err(e),
            }?;

            // Connect to ratmand and generate a new address.
            let ipc = RatmanIpc::default().await?;
            let addr = ipc.address();

            // Write it to the "addr" file.
            async_std::fs::write(&addr_path, addr.to_string().as_bytes()).await?;

            Ok(addr)
        }

        // Something else went wrong, eg. the file has the wrong permissions set.
        // Don't attempt to clobber it; tell the user and let them figure it out.
        Err(e) => Err(e.into()),
    }
}
