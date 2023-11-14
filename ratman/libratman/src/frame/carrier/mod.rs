#![allow(unused)]

mod announce;
mod header;
mod manifest;

pub mod generate;
pub mod parse;

////// Frame type exports
pub use announce::*;
pub use header::*;
pub use manifest::*;

////// Expose the generator and parser APIs for other types
pub use parse::{take_address, IResult as ParserResult};

pub(self) fn random_payload(size: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut buf = vec![0; size];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.into()
}

pub mod modes {
    pub fn str_name(mode: u16) -> &'static str {
        match mode {
            ANNOUNCE => "announce frame",
            DATA => "ERIS block data frame",
            MANIFEST => "ERIS root manifest frame",
            val => "[UNKNOWN]",
        }
    }

    pub const ANNOUNCE: u16 = 2;
    pub const DATA: u16 = 4;
    pub const MANIFEST: u16 = 5;
}
