// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    types::identifiers::{id::Id, ID_LEN},
    Result,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[cfg(feature = "metrics")]
use prometheus_client::encoding::text::Encode;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address(Id);

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Address {
    /// Expand a piece of input into an address
    ///
    /// Currently this only discards the private key section
    pub fn expand_input(input: &Vec<u8>) -> Self {
        warn!("Address::expand_input will change semantically in a future version!");
        Self(Id::with_digest(input))
    }

    #[deprecated]
    pub fn random() -> Self {
        warn!("Generating a random Address is deprecated and will be removed in a future version of this crate!");
        Self(Id::random())
    }

    /// Parse a string representation of an address
    pub fn from_string(s: &String) -> Self {
        Self(Id::from_string(s))
    }

    /// Create an address from a byte slice
    ///
    /// If the slice is not long enough (or too long), an appropriate
    /// error will be returned, instead of panicking.
    pub fn try_from_bytes(buf: &[u8]) -> Result<Self> {
        Id::try_from_bytes(buf).map(|id| Self(id))
    }

    /// Expand a slice of bytes into an address
    ///
    /// This function will panic if not enough bytes were provided
    pub fn from_bytes(buf: &[u8]) -> Self {
        Self(Id::from_bytes(buf))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Copy the contents out as a slice
    #[inline]
    pub fn slice(&self) -> [u8; ID_LEN] {
        self.0.slice()
    }
}

/// Implement RAW `From` binary array
impl From<[u8; ID_LEN]> for Address {
    fn from(i: [u8; ID_LEN]) -> Self {
        Self(i.into())
    }
}

/// Implement RAW `From` binary (reference) array
impl From<&[u8; ID_LEN]> for Address {
    fn from(i: &[u8; ID_LEN]) -> Self {
        Self(i.clone().into())
    }
}

#[cfg(feature = "metrics")]
impl Encode for Address {
    fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.encode(w)
    }
}
