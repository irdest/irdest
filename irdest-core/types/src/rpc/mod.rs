//! RPC support module
//!
//! `libqaul-types` by itself simply provides a set of Rust types used
//! across libqaul and associated crates and services.  In order to
//! support encoding and decoding these types for the RPC layer, you
//! can enable the RPC module, which provides a set of builder
//! functions to transform types.

#[cfg(test)]
mod tests;

use crate::{
    error::Error,
    messages::{IdType, Message, Mode, MsgId, MsgQuery},
    services::Service,
    users::{UserAuth, UserProfile, UserUpdate},
    Identity,
};
use alexandria_tags::TagSet;
use serde::{Deserialize, Serialize};

pub const ADDRESS: &'static str = "org.irdest.core";

/// Capabilities are functions that can be executed on a remote
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "context", rename_all = "kebab-case")]
pub enum Capabilities {
    Users(UserCapabilities),
    Services(ServiceCapabilities),
    Messages(MessageCapabilities),
    Contacts(ContactCapabilities),
    UnregisterSub(Identity),
}

impl Capabilities {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Invalid type: can't be made into json!")
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

/// User scope libqaul functions
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "kebab-case")]
pub enum UserCapabilities {
    List,
    ListRemote,
    IsAuthenticated { auth: UserAuth },
    Create { pw: String },
    Delete { auth: UserAuth },
    ChangePw { auth: UserAuth, new_pw: String },
    Login { id: Identity, pw: String },
    Logout { auth: UserAuth },
    Get { id: Identity },
    Update { auth: UserAuth, update: UserUpdate },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "kebab-case")]
pub enum ServiceCapabilities {
    /// Register a new service
    ///
    /// This type _actually_ creates a subscription that is mapped to
    /// a user function.
    Register { name: String },
    /// Unregister a service.  This MUST also remove the subscription
    /// associated to this service
    Unregister { name: String, sub: Identity },
    Insert {
        auth: UserAuth,
        service: String,
        key: String,
        value: Vec<u8>,
    },
    Delete {
        auth: UserAuth,
        service: String,
        key: String,
    },
    Query {
        auth: UserAuth,
        service: String,
        key: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "kebab-case")]
pub enum MessageCapabilities {
    Send {
        auth: UserAuth,
        mode: Mode,
        id_type: IdType,
        service: Service,
        tags: TagSet,
        payload: Vec<u8>,
    },
    Subscribe {
        auth: UserAuth,
        service: Service,
        tags: TagSet,
    },
    Query {
        auth: UserAuth,
        service: Service,
        query: MsgQuery,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "kebab-case")]
pub enum ContactCapabilities {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "context", rename_all = "kebab-case")]
pub enum Reply {
    Users(UserReply),
    Message(MessageReply),
    Service(ServiceReply),

    /// A special reply type that handles registering subscriptions
    Subscription(SubscriptionReply),
    /// A special reply type that wraps all error codes
    Error(Error),
}

impl Reply {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Invalid type: can't be made into json!")
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum UserReply {
    List(Vec<UserProfile>),
    Authenticated(bool),
    Auth(UserAuth),
    Ok,
    Profile(UserProfile),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum MessageReply {
    Ok,
    Message(Message),
    MsgId(MsgId),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum SubscriptionReply {
    Ok(Identity),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum ServiceReply {
    Ok,
    /// Returned when registering a service.  The sub_id MUST be
    /// registered as a Subscription over `ServiceEvent`
    Register {
        sub: Identity,
    },
    Query {
        key: String,
        val: Vec<u8>,
    },
}
