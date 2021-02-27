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

// pub use error::ConvertError;
// pub(crate) use error::try_read;

use crate::{
    messages::{IdType, Mode, MsgQuery},
    services::Service,
    users::{UserAuth, UserUpdate},
    Identity,
};
use alexandria_tags::TagSet;

/// Capabilities are functions that can be executed on a remote
pub enum Capabilities {
    Users(UserCapabilities),
    Services(ServiceCapabilities),
    Messages(MessageCapabilities),
    Contacts(ContactCapabilities),
}

/// User scope libqaul functions
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

pub enum ServiceCapabilities {}

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

pub enum ContactCapabilities {}
