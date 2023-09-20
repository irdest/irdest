#![allow(unused)]

mod announce;
mod carrier;
mod generate;
mod manifest;
mod parse;

////// Frame type exports
pub use announce::*;
pub use carrier::*;
pub use manifest::*;
use rand::RngCore;

pub(self) fn random_payload(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.into()
}

pub mod modes {
    pub const ANNOUNCE: u16 = 0;
    pub const DATA: u16 = 2;
    pub const MANIFEST: u16 = 3;
    pub const ROUTER_HANDSHAKE: u16 = 4;
}
