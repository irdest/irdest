#![allow(unused)]

mod bincode_frame;
mod hand_frame;
mod nom;

use crate::Result;
use byteorder::{BigEndian, ByteOrder};

pub(self) fn u16_to_big_endian(val: u16) -> [u8; 2] {
    let mut v = [0; 2];
    BigEndian::write_u16(&mut v, val);
    v
}

/// A trait that implements the Irdest frame encoding format
///
/// This trait by itself does nothing, and doesn't integrate with
/// serde (at the moment).  Check the MREP specification for how
/// different payloads must be encoded.
pub trait FrameEncoder: Sized {
    fn encode(self) -> Vec<u8>;
    fn decode(v: &Vec<u8>) -> Result<Self>;
}

////// Actual exports
mod carrier;
pub use carrier::CarrierFrame;
