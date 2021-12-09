//! I/O persistence module

pub(self) mod encrypted_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/encrypted_capnp.rs"));
}

pub(self) mod chunk_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/chunk_capnp.rs"));
}

pub(crate) mod format;

mod wire;
pub(crate) use wire::{Decode, Encode};

mod sync;
pub use sync::Sync;

mod cfg;
pub use cfg::legacy as versions;
pub use cfg::Config;
