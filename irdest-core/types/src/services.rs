//! Application service management and utilities
//!
//! An application using libqaul is called a "service". qaul.net (the
//! application) is simply a collection of services that expose a
//! common UI for users to interact with each other.  A service
//! doesn't need to be user-facing, or have a UI.
//!
//! Via the [`qrpc`] message bus it is possible for arbitrary
//! processes to interact with other services, and libqaul instances.
//! Because libqaul implements encrypted at-rest storage, this
//! mechanism is exposed to services via this API.  This way your
//! application can't accidentally leak user metadata.
//!
//! [`qrpc`]: https://docs.qaul.net/api/qrpc-sdk/index.html

use std::fmt::{self, Display};

use crate::users::UserAuth;
use serde::{Deserialize, Serialize};

/// Represents a service using irdest
///
/// Via this type it's possible to either perform actions as a
/// particular survice, or none, which means that all service's events
/// become available.  While this is probably not desirable (and
/// should be turned off) in most situations, this way a user-level
/// service can do very powerful things with the "raw" netork traffic
/// of an irdest network.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Service {
    /// Get access to all service's events
    // One of the three most common passwords, you know?
    God,
    /// Service by domain qualified name (e.g. `net.qaul.chat`)
    Name(String),
}

impl<T> From<T> for Service
where
    T: Into<String>,
{
    fn from(t: T) -> Self {
        Self::Name(t.into())
    }
}

/// Event type that can be sent to services to react to state changes
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum ServiceEvent {
    /// A user session was started
    Open(UserAuth),
    /// A user session was ended
    Close(UserAuth),
}

impl ServiceEvent {
    pub fn tt(&self) -> String {
        match self {
            Self::Open(UserAuth(id, _)) => format!("OpenEvent({})", id),
            Self::Close(UserAuth(id, _)) => format!("CloseEvent({})", id),
        }
    }
}

/// A 2-String tuple used for data indexing
///
/// A `StoreKey` can be created from Strings, using the `#` symbol to
/// separate the namespace and key parts.  To access either parts of
/// the key, use the appropriate functions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreKey(String, String);

impl StoreKey {
    /// Create a StoreKey with explicit namespace and key
    pub fn new<Ns: Into<String>, Key: Into<String>>(ns: Ns, key: Key) -> Self {
        Self(ns.into(), key.into())
    }

    /// Create a StoreKey with empty namespace
    pub fn no_namespace<S: Into<String>>(key: S) -> Self {
        Self("".into(), key.into())
    }

    /// Return the namespace section of the StoreKey
    pub fn namespace(&self) -> &String {
        &self.0
    }

    /// Return the key section of the StoreKey
    pub fn key(&self) -> &String {
        &self.1
    }
}

impl From<String> for StoreKey {
    fn from(s: String) -> Self {
        let mut v: Vec<_> = s.split('#').collect();

        if v.len() == 1 {
            Self("".into(), v.remove(0).into())
        } else {
            Self(v.remove(0).into(), v.remove(0).into())
        }
    }
}

impl From<&str> for StoreKey {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl Display for StoreKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // No `#` must be present in namespaces or keys
        assert!(!self.namespace().contains("#"));
        assert!(!self.key().contains("#"));

        // Then just concat them together
        write!(f, "{}", format!("{}#{}", self.namespace(), self.key()))
    }
}
