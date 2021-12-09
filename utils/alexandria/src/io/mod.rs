//! I/O persistence module

pub(self) mod proto {
    include!(concat!(env!("OUT_DIR"), "/io/proto_gen/mod.rs"));
}

mod cfg;
mod chunk;
mod encrypted;
mod error;
mod sync;
mod wire;

pub(crate) mod format;
pub(crate) use wire::{Decode, Encode};

pub use cfg::legacy as versions;
pub use cfg::Config;
pub use sync::Sync;
