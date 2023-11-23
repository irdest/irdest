//! Carrier frame format types

mod announce;
mod header;
mod manifest;

////// Frame type exports
pub use announce::*;
pub use header::*;
pub use manifest::*;

////// Expose the generator and parser APIs for other types
pub use crate::frame::parse::{take_address, IResult as ParserResult};

#[cfg(test)]
pub fn random_payload(size: usize) -> Vec<u8> {
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
            _ => "[UNKNOWN]",
        }
    }

    pub const ANNOUNCE: u16 = 2;
    pub const DATA: u16 = 4;
    pub const MANIFEST: u16 = 5;
}
