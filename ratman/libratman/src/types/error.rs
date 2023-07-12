// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::types::Address;
use async_std::io;

pub type Result<T> = std::result::Result<T, RatmanError>;

/// A central error facade for Ratman
///
/// Sub-scopes are handled via their own error types that are then
/// nested into this one.  Importantly, `Result` always refers to
/// `Result<T, RatmanError>` to keep the hand-up chain of errors
/// consistent.
///
// TODO(design): how to differentiate between errors that are fatal
// and those that are not?
#[derive(Debug, thiserror::Error)]
pub enum RatmanError {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
    #[cfg(feature = "proto")]
    #[error("failed to parse base encoding: {}", 0)]
    Proto(#[from] protobuf::ProtobufError),
    #[cfg(feature = "client")]
    #[error("a client API error occurred: {}", 0)]
    ClientApi(crate::ClientError),
    #[cfg(feature = "netmod")]
    #[error("a netmod error occurred: {}", 0)]
    Netmod(crate::NetmodError),
    #[error("failed to de-sequence a series of frames")]
    DesequenceFault,
    #[error("the given address '{}' is unknown to this router!", 0)]
    NoSuchAddress(Address),
    #[error("the address '{}' already exists!", 0)]
    DuplicateAddress(Address),
}

impl From<RatmanError> for io::Error {
    fn from(e: RatmanError) -> Self {
        match e {
            RatmanError::Io(e) => e,
            e => panic!("unexpected IPC error: {}", e),
        }
    }
}
