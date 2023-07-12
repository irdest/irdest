use std::net::AddrParseError;

/// Any error that can occur when interacting with a netmod driver
#[derive(Clone, Debug, thiserror::Error)]
pub enum NetmodError {
    #[error("the requested operation is not supported by the netmod")]
    NotSupported,
    #[error("frame is too large to send through this channel")]
    FrameTooLarge,
    #[error("peering connection was suddenly lost mid-transfer")]
    ConnectionLost,
    #[error("the provided peer '{}' was invalid!", 0)]
    InvalidPeer(String),
    /// An error type for a netmod that tries to bind any resource
    #[error("failed to setup netmod bind: {}", 0)]
    InvalidBind(String),
}

impl From<AddrParseError> for NetmodError {
    fn from(err: AddrParseError) -> Self {
        Self::InvalidBind(err.to_string())
    }
}
