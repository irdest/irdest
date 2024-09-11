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
    // !!! CONSULT THE MREP SPECIFICATION BEFORE ADDING NEW MESSAGE TYPES !!!
    
    pub fn str_name(mode: u16) -> &'static str {
        match mode {
            ANNOUNCE => "announce frame",
            DATA => "ERIS block data frame",
            MANIFEST => "ERIS root manifest frame",
            ROUTER_PEERING => "Router-to-Router introduction",
            _ => "[UNKNOWN]",
        }
    }

    // 4-7 are reserved for Announcement types
    pub const ANNOUNCE: u16 = 4;

    // 8 - are main data payloads
    pub const DATA: u16 = 8;
    pub const MANIFEST: u16 = 9;

    // The set of router-router peering protocols are 64-127
    pub const ROUTER_PEERING: u16 = 64;
}

////////////////// SOME TESTS

#[test]
fn full_announce_frame() {
    use crate::{
        frame::{FrameGenerator, FrameParser},
        types::{Address, InMemoryEnvelope},
    };

    let origin = OriginDataV1::now();
    let origin_signature = [1; 64];

    // Create a full announcement and encode it
    let af = AnnounceFrame::V1(AnnounceFrameV1 {
        origin,
        origin_signature,
        route: RouteDataV1 {
            mtu: 0,
            size_hint: 0,
        },
    });

    let mut afb = vec![];
    af.clone().generate(&mut afb).unwrap();
    println!("abf len: {}", afb.len());

    let h = CarrierFrameHeader::new_announce_frame(Address::random(), afb.len() as u16);

    let mut hb = vec![];
    h.clone().generate(&mut hb).unwrap();
    println!("hb len: {}", hb.len());

    ////// Decoding time

    let in_memory_env = InMemoryEnvelope::from_header_and_payload(h.clone(), afb.clone()).unwrap();

    let (r, cfhp) = CarrierFrameHeader::parse(&hb).unwrap();
    assert_eq!(r, crate::frame::EMPTY);

    assert_eq!(cfhp.unwrap(), h);

    let (r2, afp) = AnnounceFrame::parse(&afb).unwrap();
    assert_eq!(r2, crate::frame::EMPTY);
    assert_eq!(afp.unwrap(), af);

    assert_eq!(hb.len(), h.get_size());

    let (r3, afp2) = AnnounceFrame::parse(&in_memory_env.get_payload_slice()).unwrap();
    assert_eq!(r3, crate::frame::EMPTY);
    assert_eq!(afp2.unwrap(), af);
}
