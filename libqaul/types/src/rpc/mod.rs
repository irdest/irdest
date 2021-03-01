//! RPC support module
//!
//! `libqaul-types` by itself simply provides a set of Rust types used
//! across libqaul and associated crates and services.  In order to
//! support encoding and decoding these types for the RPC layer, you
//! can enable the RPC module, which provides a set of builder
//! functions to transform types.

// mod util;
// mod error;
// mod users;

#[cfg(test)]
mod tests;

// pub use error::ConvertError;
// pub(crate) use error::try_read;

use crate::{
    error::Error,
    messages::{IdType, Mode, MsgQuery},
    services::Service,
    users::{UserAuth, UserProfile, UserUpdate},
    Identity,
};
use alexandria_tags::TagSet;
use serde::{Deserialize, Serialize};

pub const ADDRESS: &'static str = "org.qaul.libqaul";

/// Capabilities are functions that can be executed on a remote
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "context")]
pub enum Capabilities {
    #[serde(rename = "users")]
    Users(UserCapabilities),
    #[serde(rename = "services")]
    Services(ServiceCapabilities),
    #[serde(rename = "messages")]
    Messages(MessageCapabilities),
    #[serde(rename = "contacts")]
    Contacts(ContactCapabilities),
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
pub enum ServiceCapabilities {}

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
#[serde(tag = "context")]
pub enum Reply {
    Users(UserReply),
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
