//! TCP ovelay specific error handling

pub type Result<T> = std::result::Result<T, Error>;

/// A generic initialisation error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the selected mode does not allow for this operation")]
    InvalidMode,
    #[error("failed to initialise socket: invalid address")]
    InvalidAddr,
    #[error("failed to send packet!")]
    FailedToSend
}

impl From<async_std::io::Error> for Error {
    fn from(e: async_std::io::Error) -> Self {
        use async_std::io::ErrorKind::*;
        match e.kind() {
            PermissionDenied | AddrInUse | AddrNotAvailable => Self::InvalidAddr,
            e => panic!("Unhandled io error: `{:?}`", e),
        }
    }
}
