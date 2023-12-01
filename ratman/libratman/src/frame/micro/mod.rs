//! A minimalist framing mechanism

mod error;
pub mod parse;

use crate::{
    frame::{parse as fparse, FrameGenerator, FrameParser},
    types::ClientAuth,
    Result,
};
use nom::IResult;

#[rustfmt::skip]
pub mod client_modes {

    //// List of mode namespaces that are available
    pub const INTRINSIC: u8 = 0x0;
    pub const ADDR: u8      = 0x1;
    pub const CONTACT: u8   = 0x2;
    pub const LINK: u8      = 0x3;
    pub const PEER: u8      = 0x4;
    pub const RECV: u8      = 0x5;
    pub const SEND: u8      = 0x6;
    pub const STATUS: u8    = 0x7;
    pub const SUB: u8       = 0x8;
    pub const CLIENT: u8    = 0x9;
    
    //// Creating new data on the network, or destroying it properly
    pub const CREATE: u8    = 0x1;
    pub const DESTROY: u8   = 0x2;

    //// Transitioning a component from inactive to active, or from
    //// active to inactive.
    pub const UP: u8        = 0x3;
    pub const DOWN: u8      = 0x4;

    //// Add and delete are reversible, and re-appliable.  No source
    //// data is being destroyed, just associations.
    pub const ADD: u8       = 0x5;
    pub const DELETE: u8    = 0x6;
    pub const MODIFY: u8    = 0x7;

    //// A bunch of modes I forgot why we needed
    pub const LIST: u8      = 0x10;
    pub const QUERY: u8     = 0x11;
    pub const ONE: u8       = 0x12;
    pub const MANY: u8      = 0x13;
    pub const FLOOD: u8     = 0x14;
    pub const FETCH: u8     = 0x15;
    pub const SYSTEM: u8    = 0x16;
    pub const OP_ADDR: u8   = 0x17;
    pub const OP_LINK: u8   = 0x18;


    /// Assemble a full mode byte from a command namespace and a
    /// compatible operator.  Not all mode encodings are valid and may
    /// be rejected by the remote.
    pub const fn make(ns: u8, op: u8) -> u16 {
        ((ns as u16) << 8) as u16 | op as u16
    }

    #[test]
    fn test_addr_create() {
        let mode = make(ADDR, CREATE);
        println!("{:#018b}", mode);
        assert_eq!(mode, 257);
    }
}

/// Metadata header for a Microframe
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct MicroframeHeader {
    pub modes: u16,
    pub auth: Option<ClientAuth>,
    pub payload_size: u32,
}

impl FrameGenerator for MicroframeHeader {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.modes.generate(buf)?;
        self.auth.generate(buf)?;
        self.payload_size.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for MicroframeHeader {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, modes) = fparse::take_u16(input)?;
        let (input, auth) = ClientAuth::parse(input)?;
        let (input, payload_size) = fparse::take_u32(input)?;

        Ok((
            input,
            Ok(MicroframeHeader {
                modes,
                auth,
                payload_size,
            }),
        ))
    }
}

/// Creates a Microframe from
///
/// - message modes
/// - an optional client auth token
/// - an optional inner message payload
pub fn encode_micro_frame<T: FrameGenerator>(
    modes: u16,
    auth: Option<ClientAuth>,
    payload: Option<T>,
) -> Result<Vec<u8>> {
    let mut payload_buf = vec![];
    match payload {
        Some(p) => p.generate(&mut payload_buf)?,
        None => {}
    };

    let header = MicroframeHeader {
        modes,
        auth,
        payload_size: payload_buf.len() as u32,
    };

    let mut complete = vec![];
    header.generate(&mut complete)?;
    complete.append(&mut payload_buf);

    Ok(complete)
}
