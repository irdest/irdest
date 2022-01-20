use async_std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
    #[error("failed to parse base encoding: {}", 0)]
    Proto(#[from] protobuf::ProtobufError),
    #[error("failed to provide correct authentication in handshake")]
    InvalidAuth,
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Io(e) => e,
            e => panic!("unexpected IPC error: {}", e),
        }
    }
}
