use super::bincode_frame::*;
use crate::client::Address;
use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use flate2::{write::DeflateEncoder, Compression};
use std::{fs::File, io::Write};

fn encode_announcement(mut pl: AnnouncePayload) -> Vec<u8> {
    let mut vec = vec![];

    // Include an overall version byte for this structure in case we
    // want to make sweeping changes at some point or have different
    // versions of announcements.
    vec.push(0x1 as u8);

    // Encode OriginData -- we introduce a simple 1-byte version
    // scheme here to keep track of different revisions of this field.
    vec.push(0x1 as u8);
    let timestamp = pl.origin.timestamp.to_rfc3339();
    println!("timestamp_length={}", timestamp.len());
    vec.append(&mut timestamp.as_bytes().to_vec());
    vec.append(&mut pl.origin_sign);

    // Encode PeerData -- we include a version number, then a single
    // null byte to indicate that the field is empty.  We leave a
    // second null byte to leave the signature field empty.
    vec.push(0x1 as u8);
    vec.push(0x0 as u8);
    vec.push(0x0 as u8);

    // Finally encode RouteData -- include the same version number,
    // then two sweet u16s
    vec.push(0x1 as u8);
    vec.append(&mut {
        let mut v = [0; 2];
        BigEndian::write_u16(&mut v, pl.route.mtu);
        v.to_vec()
    });
    vec.append(&mut {
        let mut v = [0; 2];
        BigEndian::write_u16(&mut v, pl.route.size_hint);
        v.to_vec()
    });

    vec
}

fn encode_base_frame(mut bf: BaseFrame) -> Vec<u8> {
    let mut vec = vec![];

    // Include a version byte
    vec.push(0x1 as u8);

    // Encode our modes as two bytes
    vec.append(&mut bf.modes.to_vec());

    // For the empty recipient we set a single zero byte
    vec.push(0x0 as u8);

    // Encode the sender address
    vec.append(&mut bf.sender.to_vec());

    // Leave two zero bytes for sequence_id and signature
    vec.push(0x0 as u8);
    vec.push(0x0 as u8);

    // Encode the payload length as a u16
    vec.append(&mut {
        let mut v = [0; 2];
        BigEndian::write_u16(
            &mut v,
            bf.payload
                .len()
                .try_into()
                .expect(&format!("payload size exceeded allowed {}", u16::MAX)),
        );
        v.to_vec()
    });

    // Finally append the payload
    vec.append(&mut bf.payload);

    vec
}

#[test]
fn hand_framer() {
    let origin = OriginData {
        timestamp: Utc::now(),
    };

    let payload = encode_announcement(AnnouncePayload {
        origin,
        origin_sign: Address::random().slice().to_vec(),

        peer: PeerData {},
        peer_sign: vec![],

        route: RouteData {
            mtu: 1024,
            size_hint: 1024,
        },
    });

    println!("payload length: {}", payload.len());
    let f = BaseFrame {
        modes: [0, 0],
        recipient: None,
        sender: Address::random().slice(),
        seq_id: None,
        signature: None,
        payload,
    };

    println!("{:?}", f);
    let base_buf = encode_base_frame(f);

    // Persist the output so we can look at it in a hex viewer
    if let Some(mut f) = File::create("./hand.bin").ok() {
        f.write_all(&base_buf).unwrap();
    }

    let mut e = DeflateEncoder::new(Vec::new(), Compression::best());
    e.write_all(&base_buf).unwrap();
    let base_compressed = e.finish().unwrap();

    println!(
        "====\n f encoded len = {}\n{:?}\n\n",
        base_buf.len(),
        base_buf
    );

    println!(
        "====\n f compressed len = {}\n{:?}\n\n",
        base_compressed.len(),
        base_compressed
    );
}
