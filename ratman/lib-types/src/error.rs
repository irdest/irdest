// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
    #[cfg(feature = "proto")]
    #[error("failed to parse base encoding: {}", 0)]
    Proto(#[from] protobuf::ProtobufError),
    #[error("failed to provide correct authentication in handshake")]
    InvalidAuth,
    #[error("failed to de-sequence a series of frames")]
    DesequenceFault,
    #[error("connection was unexpectedly dropped")]
    ConnectionLost,
    #[error("frame is too large to send through this channel")]
    FrameTooLarge,
    #[error("operation not supported")]
    NotSupported,
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Io(e) => e,
            e => panic!("unexpected IPC error: {}", e),
        }
    }
}
