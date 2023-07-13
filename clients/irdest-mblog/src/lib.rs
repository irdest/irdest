mod lookup;

use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use libratman::{
    client::RatmanIpc,
    types::{Address, Id, Message as RatmanMessage, Recipient, TimePair},
};
use protobuf::Message as _;
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
    pub fn from_ratmsg(ratmsg: &RatmanMessage) -> Self {
        Self {
            header: Header::message(&ratmsg),
            payload: ratmsg.get_payload(),
        }
    }

    pub fn parse_proto(data: &[u8]) -> Result<Self> {
        Ok(proto::db::Envelope::parse_from_bytes(data)?.into())
    }

    pub fn into_proto(self) -> proto::db::Envelope {
        proto::db::Envelope {
            id: self.header.id.as_bytes().into(),
            time_ns: self.header.time.timestamp_nanos(),
            sender: self
                .header
                .sender
                .map(|v| v.as_bytes().into())
                .unwrap_or_default(),
            recipient_type: match &self.header.recipient {
                &Recipient::Flood(_) => proto::db::RecipientType::RECIPIENT_FLOOD,
                &Recipient::Standard(_) => proto::db::RecipientType::RECIPIENT_STANDARD,
            },
            recipients: match self.header.recipient {
                // Don't store the address if it's Flood(NAMESPACE); it's implicit.
                Recipient::Flood(a) if a.as_bytes() == &NAMESPACE[..] => vec![].into(),
                Recipient::Flood(a) => vec![a.as_bytes().into()].into(),
                Recipient::Standard(aa) => aa
                    .iter()
                    .map(|a| a.as_bytes().into())
                    .collect::<Vec<_>>()
                    .into(),
            },
            payload: self.payload,
            ..Default::default()
        }
    }

    pub fn into_message(self) -> Result<Message> {
        let p = proto::feed::Message::parse_from_bytes(&self.payload[..])?;
        Ok(Message {
            header: self.header,
            payload: p.payload.ok_or(anyhow!("message has no payload?"))?.into(),
        })
    }
}

impl From<proto::db::Envelope> for Envelope {
    fn from(v: proto::db::Envelope) -> Self {
        Self {
            header: Header {
                id: Id::from_bytes(&v.id[..]),
                time: chrono::Utc.timestamp_nanos(v.time_ns),
                sender: if v.sender.len() > 0 {
                    Some(Address::from_bytes(&v.sender))
                } else {
                    None
                },
                recipient: match v.recipient_type {
                    proto::db::RecipientType::RECIPIENT_FLOOD => {
                        // Note: We don't store an address if the destination is Flood(NAMESPACE),
                        // and assume that any Flood message without an address is for NAMESPACE,
                        // as this saves 32b + overhead/message for nearly 100% of all messages.
                        //
                        // As of writing, we discard all incoming messages for other namespaces,
                        // but this code will correctly handle them anyway if one is in the DB.
                        Recipient::Flood(Address::from_bytes(
                            v.recipients
                                .first()
                                .map(|b| &b[..])
                                .unwrap_or(&NAMESPACE[..]),
                        ))
                    }
                    proto::db::RecipientType::RECIPIENT_STANDARD => Recipient::Standard(
                        v.recipients
                            .iter()
                            .map(|b| Address::from_bytes(&b[..]))
                            .collect(),
                    ),
                },
            },
            payload: v.payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub id: Id,
    pub time: DateTime<Utc>,
    pub sender: Option<Address>,
    pub recipient: Recipient,
}

impl Header {
    pub fn message(msg: &RatmanMessage) -> Self {
        Self {
            id: msg.get_id(),
            time: msg.get_time().local(),
            sender: Some(msg.get_sender()),
            recipient: msg.get_recipient(),
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            id: Id::random(),
            time: TimePair::sending().local(),
            sender: None,
            recipient: Recipient::Flood(NAMESPACE.into()),
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

    // This is a terrible terrible terrible terrible hack around the
    // way that the mblog database works.  This doesn't make much
    // sense but until we can refactor the UI to be easier to handle
    // first-run tasks, this will have to do.
    pub fn generate_intro(addr: Address) -> RatmanMessage {
        let mblog_msg = Message::new(Post {
            nick: addr.to_string(),
            topic: "/net/irdest/welcome".into(),
            text: format!(
                "Welcome to Irdest mblog, a decentralised usenet-style blogging platform!

You can create messages for different topics and subscribe to them.  \
That user identifier you see at the top of this message is your address (it should be '{}').

Why don't you say hello? :)",
                addr
            ),
        });

        let payload = mblog_msg.into_proto().write_to_bytes().unwrap();
        let mut time = TimePair::sending();
        time.receive();

        RatmanMessage::received(
            Id::random(),
            addr,
            Recipient::Flood(NAMESPACE.into()),
            payload,
            time.local().to_string(),
            vec![],
        )
        .into()
    }

    pub fn into_proto(self) -> proto::feed::Message {
        proto::feed::Message {
            payload: Some(self.payload.into()),
            ..Default::default()
        }
    }

    pub fn as_post(&self) -> &Post {
        match self.payload {
            Payload::Post(ref p) => p,
        }
    }
}

impl TryFrom<&RatmanMessage> for Message {
    type Error = anyhow::Error;

    fn try_from(msg: &RatmanMessage) -> Result<Self, Self::Error> {
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
pub async fn load_or_create_addr() -> Result<(bool, Address)> {
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
        Ok(s) => Ok((false, Address::from_string(&s))),

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

            Ok((true, addr))
        }

        // Something else went wrong, eg. the file has the wrong permissions set.
        // Don't attempt to clobber it; tell the user and let them figure it out.
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libratman::types::Recipient;

    #[test]
    // The single most common recipient is Flood(NAMESPACE), so we special-case that:
    // - Envelope->Proto should leave the recipient address field empty.
    // - Proto->Envelope should decode a missing Flood scope as NAMESPACE.
    fn test_envelope_proto_flood_our_namespace() {
        let envl = Envelope {
            header: Header::default(),
            payload: vec![],
        };
        assert_eq!(envl.header.recipient, Recipient::Flood(NAMESPACE.into()));

        let penvl = envl.clone().into_proto();
        assert_eq!(
            penvl.recipient_type,
            super::proto::db::RecipientType::RECIPIENT_FLOOD
        );
        assert_eq!(penvl.get_recipients(), Vec::<Vec<u8>>::new());

        let envl2: Envelope = penvl.into();
        assert_eq!(envl, envl2);
    }

    #[test]
    // Encoding a message to Flood(something else) should round-trip that namespace.
    // This currently can't actually happen, but we should handle it anyway.
    fn test_envelope_proto_flood_other_namespace() {
        let mut envl = Envelope {
            header: Header::default(),
            payload: vec![],
        };
        let ns: Vec<u8> = NAMESPACE.iter().rev().copied().collect();
        envl.header.recipient = Recipient::Flood(Address::from_bytes(&ns));

        let penvl = envl.clone().into_proto();
        assert_eq!(
            penvl.recipient_type,
            super::proto::db::RecipientType::RECIPIENT_FLOOD
        );
        assert_eq!(penvl.get_recipients(), vec![ns]);

        let envl2: Envelope = penvl.into();
        assert_eq!(envl, envl2);
    }

    #[test]
    // Make sure we can encode recipient=Standard(*) messages to a single recipient.
    fn test_envelope_proto_standard_one() {
        let mut envl = Envelope {
            header: Header::default(),
            payload: vec![],
        };
        let rcpt = Address::random();
        envl.header.recipient = Recipient::Standard(vec![rcpt]);

        let penvl = envl.clone().into_proto();
        assert_eq!(
            penvl.recipient_type,
            super::proto::db::RecipientType::RECIPIENT_STANDARD
        );
        assert_eq!(penvl.get_recipients(), vec![Vec::from(rcpt.as_bytes())]);

        let envl2: Envelope = penvl.into();
        assert_eq!(envl, envl2);
    }

    #[test]
    // Make sure we can encode recipient=Standard(*) messages to a multiple recipients.
    fn test_envelope_proto_standard_multi() {
        let mut envl = Envelope {
            header: Header::default(),
            payload: vec![],
        };
        let rcpt1 = Address::random();
        let rcpt2 = Address::random();
        let rcpt3 = Address::random();
        envl.header.recipient = Recipient::Standard(vec![rcpt1, rcpt2, rcpt3]);

        let penvl = envl.clone().into_proto();
        assert_eq!(
            penvl.recipient_type,
            super::proto::db::RecipientType::RECIPIENT_STANDARD
        );
        assert_eq!(
            penvl.get_recipients(),
            vec![
                Vec::from(rcpt1.as_bytes()),
                Vec::from(rcpt2.as_bytes()),
                Vec::from(rcpt3.as_bytes())
            ]
        );

        let envl2: Envelope = penvl.into();
        assert_eq!(envl, envl2);
    }
}
