use fjall::{Config, Keyspace};
use libratman::{
    frame::carrier::{modes, CarrierFrameHeader, CarrierFrameHeaderV1},
    types::{Address, Ident32, Recipient, SequenceIdV1},
};
use tempdir::TempDir;

use super::{page::SerdeFrameType, types::FrameData, Journal};

fn setup_db() -> Keyspace {
    Keyspace::open(Config::new(
        TempDir::new("journal")
            .unwrap()
            .into_path()
            .join("test.jrnl"),
    ))
    .unwrap()
}

#[test]
fn insert_get_frames() {
    let db = setup_db();
    let journal = Journal::new(db).unwrap();

    let header = CarrierFrameHeader::V1(CarrierFrameHeaderV1::new(
        modes::DATA,
        Address::random(),
        Some(Recipient::Namespace(Address::random())),
        Some(SequenceIdV1 {
            hash: Ident32::random(),
            num: 0,
            max: 0,
        }),
        None,
        None,
        0,
    ));

    let frame_id = header.get_seq_id().unwrap().hash;
    let frame_data = FrameData {
        header: header.into(),
        payload: vec![],
    };

    journal
        .frames
        .insert(frame_id.to_string(), &frame_data)
        .unwrap();
    journal.seen_frames.insert(&frame_id).unwrap();

    let recovered_event = journal.frames.get(&frame_id.to_string()).unwrap();
    assert_eq!(Some(frame_data), recovered_event);
}
