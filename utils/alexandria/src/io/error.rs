use protobuf::ProtobufError;

pub type Result<T> = std::result::Result<T, IoError>;

/// A fatal wrapper error type for I/O operations
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(std::io::Error),
    #[error("failed to parse base encoding: {}", 0)]
    Encoding(#[from] ProtobufError),
}
