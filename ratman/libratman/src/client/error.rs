/// Client API errors beetw Ratman and an application
///
/// The client API consists of an authentication handshake, message
/// sending and receiving, and simple state management (for
/// subscriptions, tokens, etc).
///
/// Importantly, more base-type errors (such as I/O and encoding) are
/// handled by [RatmanError](crate::RatmanError) instead!
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to provide correct authentication in handshake")]
    InvalidAuth,
    #[error("connection was unexpectedly dropped")]
    ConnectionLost,
    #[error("operation not supported")]
    NotSupported,
    #[error("requested an unknown address")]
    NoAddress,
    #[error("address already exists in routing table")]
    DuplicateAddress,
}
