//! Error handling types

pub use ratman_types::Error;

/// A `netmod` specific `Result` wrapper
pub type Result<T> = std::result::Result<T, Error>;
