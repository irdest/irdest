//! Ratman client & interface library
//!
//! Ratman is a packet router daemon, which can either run
//! stand-alone, or be embedded into existing applications.  This
//! library provides type definitions, utilities, and interfaces to
//! interact with the Ratman router core.
//!
//! This library can be used in two different ways (not mutually
//! exclusive, although doing both at the same time would be a bit
//! weird.  But we won't judge you).
//!
//! 1. To write a ratman-client application.  The main types for this
//! can be found in `api`.
//!
//! 2. To write a ratman-netmod driver.  The main trait type to
//! implement can be found in `endpoint`.

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

// Re-export existing errors at the root to make them more convenient
// to access.  Importantly errors are name-spaced while results are
// not.  A result MUST always be of type Result<T, RatmanError>.
pub use types::error::{
    BlockError, ClientError, EncodingError, MicroframeError, NetmodError, NonfatalError,
    RatmanError, Result, ScheduleError,
};

// Re-export tokio and futures crates to share async abstractions
pub use futures;
pub use tokio;
