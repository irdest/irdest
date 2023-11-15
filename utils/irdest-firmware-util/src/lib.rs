// TODO: can we set no_std only when the "std" feature is not set?
// #[cfg(not(feature = "std"))] // <-- this doesn't seem to work?
// #![no_std]
#![allow(warnings)]
use libratman::types::{Address, InMemoryEnvelope, Recipient};
use std::io::{Read, Write};

/// Encode a frame for basic wire formats
pub fn encode_frame<T: Write>(
    _stream: &mut T,
    _f: &InMemoryEnvelope,
) -> Result<(), std::io::Error> {
    // stream.write_all(f.sender.as_bytes())?;

    // // Prepend the recipient with a magic byte to select between
    // // "standard" and "flood" recipient types
    // match f.recipient {
    //     Recipient::Standard(ref vec) => {
    //         stream.write_all(&[0x13])?;
    //         stream.write_all(vec.get(0).unwrap().as_bytes())?;
    //     }
    //     Recipient::Flood(ns) => {
    //         stream.write_all(&[0x12])?;
    //         stream.write_all(ns.as_bytes())?;
    //     }
    // }

    // // Encode the SeqData struct one big-endian number at a time.
    // // TODO: this is awful and all this information is probably not
    // // even really needed...
    // let seq_encoded: SeqDataEncoded = (&f.seq).into();
    // stream.write_all(&seq_encoded.num)?;
    // stream.write_all(&seq_encoded.sig)?;
    // stream.write_all(&seq_encoded.seqid)?;
    // stream.write_all(&seq_encoded.next)?;

    // // FINALLY write the payload
    // let payload_len: [u8; 4] = (f.payload.len() as u32).to_be_bytes();
    // stream.write_all(payload_len.as_slice())?;
    // stream.write_all(&f.payload.as_slice())?;

    // Happy camper
    Ok(())
}

fn read_exactly(size: usize, reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

// TODO: add better errors?
pub fn decode_frame<T: Read>(stream: &mut T) -> Result<InMemoryEnvelope, std::io::Error> {
    // Read 32 bytes
    let sender_buf = read_exactly(32, stream)?;
    let sender = Address::from_bytes(&sender_buf);

    // Read 33 bytes
    let recipient_match = read_exactly(1, stream)?;
    let recipient_buf = read_exactly(32, stream)?;
    let recipient = match recipient_match[0] {
        0x13 => Recipient::Address(Address::from_bytes(&recipient_buf)),
        0x12 => Recipient::Namespace(Address::from_bytes(&recipient_buf)),
        code => panic!("Invalid recipient code: {}", code), // TODO: don't panic here
    };

    // Read 61 bytes
    let num = read_exactly(4, stream)?;
    let sig = read_exactly(16, stream)?;
    let seqid = read_exactly(32, stream)?;
    let next = read_exactly(9, stream)?;

    // let seq = SeqData::from(unsafe {
    //     SeqDataEncoded {
    //         num: num.try_into().unwrap_unchecked(),
    //         sig: sig.try_into().unwrap_unchecked(),
    //         seqid: seqid.try_into().unwrap_unchecked(),
    //         next: next.try_into().unwrap_unchecked(),
    //     }
    // });

    let payload_len_buf = read_exactly(4, stream)?;
    let payload_len = u32::from_be_bytes(unsafe { payload_len_buf.try_into().unwrap_unchecked() });
    let payload = read_exactly(payload_len as usize, stream)?;

    todo!()

    // Ok(InMemoryEnvelope {
    //     sender,
    //     recipient,
    //     seq,
    //     payload,
    // })
}

// #[test]
// fn in_and_out() {
//     let f = InMemoryEnvelope::dummy();

//     let mut buffer = vec![];
//     encode_frame(&mut buffer, &f).unwrap();

//     let f2 = decode_frame(&mut buffer.as_slice()).unwrap();
//     assert_eq!(f, f2);
// }
