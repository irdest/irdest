use crate::{client::Address, types::proto::frame::*};
use chrono::Utc;
use flate2::{write::DeflateEncoder, Compression};
use protobuf::Message as _;
use std::{io::Write, fs::File};

#[test]
pub fn protobuf_framer() {
    let mut base = Frame::new();

    let mut announce_payload = AnnouncePayload::new();
    let mut origin_data = OriginData::new();
    origin_data.set_timestamp(Utc::now().to_rfc3339());
    announce_payload.set_origin_data(origin_data);
    announce_payload.set_origin_signature(Address::random().slice().to_vec());
    let mut route_data = RouteData::new();
    route_data.set_mtu(1024);
    route_data.set_size_hint(1024);
    announce_payload.set_route_data(route_data);

    let payload_buf = announce_payload.write_to_bytes().unwrap();
    println!("payload length: {}", payload_buf.len());

    base.set_modes(0);
    base.set_sender(Address::random().slice().to_vec());
    base.set_payload(payload_buf);

    let base_buf = base.write_to_bytes().unwrap();
    println!("{:?}", base);

    // Persist the output so we can look at it in a hex viewer
    if let Some(mut f) = File::create("./proto.bin").ok() {
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
