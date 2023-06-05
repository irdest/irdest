//! Error handling types

// TODO: uuuuuh, what is this ??
pub use crate::types::Error;

/// A `netmod` specific `Result` wrapper
pub type Result<T> = std::result::Result<T, Error>;
