use async_std::io;

pub(crate) type Result<T> = std::result::Result<T, DaemonError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DaemonError {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
}
