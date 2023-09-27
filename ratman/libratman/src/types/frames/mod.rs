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

////// Expose the generator and parser APIs for other types
pub use generate::FrameGenerator;
pub use parse::{FrameParser, IResult as ParserResult};

use rand::RngCore;

pub(self) fn random_payload(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.into()
}

pub mod modes {
    pub const ANNOUNCE: u16 = 2;
    pub const DATA: u16 = 4;
    pub const MANIFEST: u16 = 5;
}
