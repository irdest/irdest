//! RPC related error handling

use std::fmt::{self, Display, Formatter};

pub type RpcResult<T> = Result<T, RpcError>;

/// A set of errors that occur when connecting to services
#[derive(Debug, Serialize, Deserialize)]
pub enum RpcError {
    /// No such service was found by the broker
    NoSuchService(String),
    /// The requested action could not be performed because the
    /// provided hash_id was invalid
    NotAuthorised,
    /// The selected recipient didn't reply within the timeout
    ///
    /// This may indicate that the requested service has crashed, is
    /// dealing with backpressure, or the broker is quietly dropping
    /// requests.
    Timeout,
    /// Tried connecting to a service that's already connected
    AlreadyRegistered,
    /// While trying to handle registration, an error occurred
    RegistryFailed,
    /// Invalid connection: performing the last operation has failed
    ConnectionFault(String),
    /// Encoding or decoding a payload failed
    EncoderFault(String),
    /// A payload other than the one expected was received
    UnexpectedPayload,
    /// Subscription can never yield more data
    SubscriptionEnded,
    /// No such subscription exists
    NoSuchSubscription,
    /// **Non fatal** Failed parsing, resume normally
    NotASubscription,
    /// Other failures encoded as an error message
    Other(String),
}

impl Display for RpcError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoSuchService(s) => format!("The requested service {} does not exist!", s),
                Self::NotAuthorised => format!("Operation denied: provided hash ID was not valid"),
                Self::Timeout => "The requested operation took too long (timeout)!".into(),
                Self::AlreadyRegistered =>
                    "Tried registering a service that is already registered".into(),
                Self::RegistryFailed =>
                    "Failed to register a service because of an invalid payload".into(),
                Self::ConnectionFault(s) => format!("I/O error: {}", s),
                Self::EncoderFault(s) => format!("Encode error: {}", s),
                Self::UnexpectedPayload => format!("Parsing encountered unexpected payload"),
                Self::SubscriptionEnded => format!("Subscription error"),
                Self::NoSuchSubscription => format!("Subscription error"),
                Self::NotASubscription => format!("C'est n'est pas une subscription"),
                Self::Other(s) => format!("Unknown error: {}", s),
            }
        )
    }
}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> Self {
        Self::ConnectionFault(e.to_string())
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(e: serde_json::Error) -> Self {
        Self::EncoderFault(format!("{}", e))
    }
}

impl From<async_std::future::TimeoutError> for RpcError {
    fn from(_: async_std::future::TimeoutError) -> Self {
        Self::Timeout
    }
}
