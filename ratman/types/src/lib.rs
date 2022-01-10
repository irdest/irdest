//! API encoding types for Ratman

mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto_gen/mod.rs"));
}

pub mod api;
pub mod message;
