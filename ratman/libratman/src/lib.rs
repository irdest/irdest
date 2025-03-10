#![doc = include_str!("../README.md")]

// We include all tracing macros to make our life easier
#[macro_use]
extern crate tracing;

// Include modules publicly
pub mod api;
pub mod chunk;
pub mod endpoint;
pub mod frame;
pub mod rt;
pub mod types;

use ed25519_dalek::{PublicKey, SecretKey};
use rand::rngs::OsRng;

// Re-export existing errors at the root to make them more convenient
// to access.  Importantly errors are name-spaced while results are
// not.  A result MUST always be of type Result<T, RatmanError>.
pub use types::error::{
    BlockError, ClientError, EncodingError, MicroframeError, NetmodError, NonfatalError,
    RatmanError, Result, ScheduleError,
};

#[cfg(feature = "daemon")]
pub use axum;
#[cfg(feature = "daemon")]
pub use axum_embed;
pub use futures;
pub use hex;
pub use tokio;
pub use tokio_stream;
pub use tokio_util;

use types::{Address, Ident32};

/// Print a log message and exit
// TODO: turn into macro
pub fn elog<S: Into<String>>(msg: S, code: u16) -> ! {
    error!("{}", msg.into());
    std::process::exit(code.into());
}

/// Get XDG_DATA_HOME from the environment
pub fn env_xdg_data() -> Option<String> {
    std::env::var("XDG_DATA_HOME").ok()
}

/// Get XDG_CONFIG_HOME from the environment
pub fn env_xdg_config() -> Option<String> {
    std::env::var("XDG_CONFIG_HOME").ok()
}

/// Create a new private/public keypair usable as a Namespace address
///
/// Include this data in all instances of your application to have access to the
/// namespace.  You can use `ipc.namespace_register()` to register this
/// namespace key pair in the local router instance.
pub fn generate_space_key() -> (Address, Ident32) {
    let secret_key = SecretKey::generate(&mut OsRng {});
    let public_key = PublicKey::from(&secret_key);

    let space_key = Ident32::from_bytes(secret_key.as_bytes());
    let space_addr = Address::from_bytes(public_key.as_bytes());

    (space_addr, space_key)
}
