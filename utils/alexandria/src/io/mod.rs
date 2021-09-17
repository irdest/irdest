//! I/O persistence module

pub(crate) mod chunk;
pub(crate) mod format;

mod wire;
pub(crate) use wire::{Decode, Encode};

mod sync;
pub use sync::Sync;

mod cfg;
pub use cfg::legacy as versions;
pub use cfg::Config;
