//! A minimalist framing mechanism

mod error;
mod generate;
pub mod parse;

pub mod types;
pub use error::Error as MicroframeError;
use tokio::io::AsyncRead;

use crate::{
    types::{
        frames::{FrameGenerator, FrameParser},
        Id,
    },
    Result,
};
use nom::IResult;

#[rustfmt::skip]
pub mod client_modes {

    //// List of mode namespaces that are available\
    pub const INTRINSIC: u8 = 0b0000_0000;
    pub const ADDR: u8      = 0b0001_0000;
    pub const CONTACT: u8   = 0b0010_0000;
    pub const LINK: u8      = 0b0011_0000;
    pub const PEER: u8      = 0b0100_0000;
    pub const RECV: u8      = 0b0101_0000;
    pub const SEND: u8      = 0b0110_0000;
    pub const STATUS: u8    = 0b0111_0000;
    pub const SUB: u8       = 0b1000_0000;
    // ... 7 more mode namespaces available


    //// Creating new data on the network, or destroying it properly
    pub const CREATE: u8    = 0b0000_1000;
    pub const DESTROY: u8   = 0b0000_1000;

    //// Transitioning a component from inactive to active, or from
    //// active to inactive.
    pub const UP: u8        = 0b0000_1000;
    pub const DOWN: u8      = 0b0000_1000;

    //// Add and delete are reversible, and re-appliable.  No source
    //// data is being destroyed, just associations.
    pub const ADD: u8       = 0b0000_0001;
    pub const DELETE: u8    = 0b0000_0010;
    pub const MODIFY: u8    = 0b0000_0011;
    pub const LIST: u8      = 0b0000_0100;
    pub const QUERY: u8     = 0b0000_0101;
    pub const ONE: u8       = 0b0000_0110;
    pub const MANY: u8      = 0b0000_0111;
    pub const FLOOD: u8     = 0b0000_1000;
    pub const FETCH: u8     = 0b0000_1001;
    pub const SYSTEM: u8    = 0b0000_1010;
    pub const OP_ADDR: u8   = 0b0000_1011;
    pub const OP_LINK: u8   = 0b0000_1100;
    // ... 3 more mode operands available


    /// Assemble a full mode byte from a command namespace and a
    /// compatible operator.  Not all mode encodings are valid and may
    /// be rejected by the remote.
    pub const fn make(ns: u8, op: u8) -> u16 {
        assert!(ns > 0b0000_1111);
        assert!(op < 0b1111_0000);
        ns as u16 & op as u16
    }
}

/// Metadata header for a Microframe
#[repr(C)]
pub struct MicroframeHeader {
    modes: u16,
    metadata: Option<Id>,
    payload_size: u32,
}

impl MicroframeHeader {
    pub async fn parse_with_reader(mut r: &mut (impl AsyncRead + Unpin)) -> Result<Self> {
        
        todo!()
    }
}

// impl FrameParser for MicroframeHeader {
//     type Output = Result<Self>;

//     fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {}
// }
