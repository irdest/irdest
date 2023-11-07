// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::types::Address;
use async_eris::BlockReference;
use async_std::io;

pub type Result<T> = std::result::Result<T, RatmanError>;

/// A central error facade for Ratman
///
/// Sub-scopes are handled via their own error types that are then
/// nested into this one.  Importantly, `Result` always refers to
/// `Result<T, RatmanError>` to keep the hand-up chain of errors
/// consistent.
///
/// When adding new category errors, make sure that the error message
/// follows the same consistent pattern of printing to "a {whatever}
/// error".  This allows for the error message to be chained in a
/// meaningful way.
// TODO(design): how to differentiate between errors that are fatal
// and those that are not?
#[derive(Debug, thiserror::Error)]
pub enum RatmanError {
    #[error("a non-fatal error {0}")]
    Nonfatal(#[from] self::NonfatalError),
    #[error("an i/o error: {0}")]
    Io(#[from] io::Error),
    // TODO: rename to Protobuf
    #[cfg(feature = "proto")]
    #[error("a base encoding error: {0}")]
    Proto(#[from] protobuf::ProtobufError),
    #[error("a frame parsing error: {0}")]
    Encoding(#[from] self::EncodingError),
    #[error("a json encoding error: {0}")]
    Json(#[from] serde_json::error::Error),
    #[cfg(feature = "client")]
    #[error("a client API error: {0}")]
    ClientApi(#[from] crate::ClientError),
    #[cfg(feature = "netmod")]
    #[error("a netmod error: {0}")]
    Netmod(#[from] crate::NetmodError),
    #[error("a block error: {0}")]
    Block(#[from] crate::BlockError),
    // #[cfg(all(feature = "daemon", target_family = "unix"))]
    // #[error("a unix system error: {0}")]
    // UnixSystem(#[from] nix::errno::Errno),
    #[error("failed to acquire state directory lock")]
    StateDirectoryAlreadyLocked,
    #[error("failed to de-sequence a series of frames")]
    DesequenceFault,
    #[error("the given address '{0}' is unknown to this router!")]
    NoSuchAddress(Address),
    #[error("the address '{0}' already exists!")]
    DuplicateAddress(Address),
    #[error("the identifier data provided was not the correct length.  Expected {0}, got {1}")]
    WrongIdentifierLength(usize, usize),
}

impl From<RatmanError> for io::Error {
    fn from(e: RatmanError) -> Self {
        match e {
            RatmanError::Io(e) => e,
            e => panic!("unexpected IPC error: {}", e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NonfatalError {
    #[error("ratman is running ephemaral mode: no data will be persisted to disk!")]
    IsEphemeral,
    #[error("the current MTU of a netmod channel is too small to fit the desired frame")]
    MtuTooSmallForFrame,
    #[error("the frame couldn't be parsed as the type it was expected to be")]
    MismatchedEncodingTypes,
    #[error("the stream or buffer didn't have any data at this time")]
    NoData,
}

#[derive(Debug, thiserror::Error)]
pub enum EncodingError {
    #[error("structure had invalid version number {0}")]
    InvalidVersion(u8),
    #[error("incoming stream could not be parsed because {0}")]
    Parsing(String),
    #[error(
        "provided frame is too large to fit into the {} size envelope: {0}",
        core::u16::MAX
    )]
    FrameTooLarge(usize),
    #[error("provided buffer did not contain any data")]
    NoData,
}

impl<T: std::fmt::Debug> From<nom::Err<T>> for EncodingError {
    fn from(nom: nom::Err<T>) -> Self {
        Self::Parsing(format!("{}", nom))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockError {
    #[error("provided data block had an invalid length: {0}")]
    InvalidLength(usize),
    #[error("provided data block integrity could not be verified (expected reference {expected}, got {got})")]
    InvalidReference {
        expected: BlockReference,
        got: BlockReference,
    },
    #[error("ERIS block decoding failed because {0}")]
    Eris(#[from] async_eris::Error),
}
