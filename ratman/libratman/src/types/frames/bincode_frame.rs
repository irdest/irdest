use crate::types::Address;
use chrono::{DateTime, Utc};
use flate2::{
    write::{DeflateEncoder, GzEncoder, ZlibEncoder},
    Compression,
};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Basic message encoding structure for sending and receiving frames
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseFrame {
    pub modes: [u8; 2],
    pub recipient: Option<[u8; 32]>,
    pub sender: [u8; 32],
    pub seq_id: Option<[u8; 16]>,
    pub signature: Option<[u8; 32]>,
    pub payload: Vec<u8>,
}

pub struct FrameBuilder {
    mtu: u16,
}

/// A BaseFrame that hasn't had its payload filled in yet
///
/// This builder structure provides the available remaining space for
/// payload via the `.max_payload()` function.  When constructing a
/// series of frames, use this function to determine how much of the
/// overall payload can be contained into this single frame.
pub struct PreallocFrame {}

impl FrameBuilder {
    pub fn new(mtu: u16) -> Self {
        Self { mtu }
    }

    pub fn announcement(self, sender: Address) -> PreallocFrame {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnnouncePayload {
    pub origin: OriginData,
    pub origin_sign: Vec<u8>,

    pub peer: PeerData,
    pub peer_sign: Vec<u8>,

    pub route: RouteData,
}

#[derive(Serialize, Deserialize)]
pub struct OriginData {
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct PeerData {}

#[derive(Serialize, Deserialize)]
pub struct RouteData {
    pub mtu: u16,
    pub size_hint: u16,
}

#[test]
fn bincode_framer() {
    let origin = OriginData {
        timestamp: Utc::now(),
    };

    let payload = bincode::serialize(&AnnouncePayload {
        origin,
        origin_sign: Address::random().slice().to_vec(),

        peer: PeerData {},
        peer_sign: vec![],

        route: RouteData {
            mtu: 1024,
            size_hint: 1024,
        },
    })
    .unwrap();

    println!("payload length: {}", payload.len());
    let f = BaseFrame {
        modes: [0, 0],
        recipient: None,
        sender: Address::random().slice(),
        seq_id: None,
        signature: None,
        payload,
    };

    let f_encoded = bincode::serialize(&f).unwrap();
    let mut e = DeflateEncoder::new(Vec::new(), Compression::best());
    e.write_all(&f_encoded).unwrap();

    let f_compressed = e.finish().unwrap();

    println!("{:?}", f);

    println!(
        "====\n f encoded len = {}\n{:?}\n\n",
        f_encoded.len(),
        f_encoded
    );
    println!(
        "====\n f compressed len = {}\n{:?}\n\n",
        f_compressed.len(),
        f_compressed
    );
}
