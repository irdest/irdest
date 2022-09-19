// TODO: can we set no_std only when the "std" feature is not set?
// #[cfg(not(feature = "std"))] // <-- this doesn't seem to work?
// #![no_std]

use ratman_types::{Frame, Identity, Recipient, SeqData, XxSignature};
use std::io::{Read, Write};

struct SeqDataEncoded {
    num: [u8; 4],    // u32
    sig: [u8; 16],   // 2x u64
    seqid: [u8; 32], // Irdest ID
    next: [u8; 9],   // u64 + Option marker
}

impl From<&'_ SeqData> for SeqDataEncoded {
    fn from(seq: &'_ SeqData) -> Self {
        let mut this = Self {
            num: seq.num.to_be_bytes(),
            sig: [0; 16],   // fill in later
            seqid: [0; 32], // fill in later
            next: [0; 9],   // fill in later,
        };

        // Encode both parts of the Signature and append them into the
        // same fixed-size buffer
        let sig_sig = seq.sig.sig.to_be_bytes();
        let sig_seed = seq.sig.seed.to_be_bytes();
        sig_sig
            .into_iter()
            .chain(sig_seed.into_iter())
            .enumerate()
            .for_each(|(idx, byte)| this.sig[idx] = byte);

        // Encode the variable length identity and copy it into a
        // fixed-size buffer
        let seqid = seq.seqid.as_bytes();
        seqid
            .into_iter()
            .enumerate()
            .for_each(|(idx, byte)| this.seqid[idx] = *byte);

        // Encode next ID as big-endian and append option marker
        match seq.next {
            Some(next) => {
                this.next[0] = true as u8;
                next.to_be_bytes()
                    .into_iter()
                    .enumerate()
                    .for_each(|(idx, byte)| this.next[idx + 1] = byte);
            }
            None => {
                this.next[1] = false as u8;
            }
        };

        this
    }
}

impl From<SeqDataEncoded> for SeqData {
    fn from(enc: SeqDataEncoded) -> Self {
        let num = u32::from_be_bytes(enc.num);
        let sig = {
            let sig = u64::from_be_bytes(unsafe { enc.sig[0..8].try_into().unwrap_unchecked() });
            let seed = u64::from_be_bytes(unsafe { enc.sig[8..16].try_into().unwrap_unchecked() });
            XxSignature { sig, seed }
        };

        let seqid = Identity::from_bytes(&enc.seqid);

        let next = {
            if enc.next[0] == true as u8 {
                Some(u64::from_be_bytes(unsafe {
                    enc.next[1..9].try_into().unwrap_unchecked()
                }))
            } else {
                None
            }
        };

        Self {
            num,
            sig,
            seqid,
            next,
        }
    }
}

/// Encode a frame for basic wire formats
pub fn encode_frame<T: Write>(stream: &mut T, f: &Frame) -> Result<(), std::io::Error> {
    stream.write_all(f.sender.as_bytes())?;

    // Prepend the recipient with a magic byte to select between
    // "standard" and "flood" recipient types
    match f.recipient {
        Recipient::Standard(ref vec) => {
            stream.write_all(&[0x13])?;
            stream.write_all(vec.get(0).unwrap().as_bytes())?;
        }
        Recipient::Flood(ns) => {
            stream.write_all(&[0x12])?;
            stream.write_all(ns.as_bytes())?;
        }
    }

    // Encode the SeqData struct one big-endian number at a time.
    // TODO: this is awful and all this information is probably not
    // even really needed...
    let seq_encoded: SeqDataEncoded = (&f.seq).into();
    stream.write_all(&seq_encoded.num)?;
    stream.write_all(&seq_encoded.sig)?;
    stream.write_all(&seq_encoded.seqid)?;
    stream.write_all(&seq_encoded.next)?;

    // FINALLY write the payload
    let payload_len: [u8; 4] = (f.payload.len() as u32).to_be_bytes();
    stream.write_all(payload_len.as_slice())?;
    stream.write_all(&f.payload.as_slice())?;

    // Happy camper
    Ok(())
}

fn read_exactly(size: usize, reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

// TODO: add better errors?
pub fn decode_frame<T: Read>(stream: &mut T) -> Result<Frame, std::io::Error> {
    // Read 32 bytes
    let sender_buf = read_exactly(32, stream)?;
    let sender = Identity::from_bytes(&sender_buf);

    // Read 33 bytes
    let recipient_match = read_exactly(1, stream)?;
    let recipient_buf = read_exactly(32, stream)?;
    let recipient = match recipient_match[0] {
        0x13 => Recipient::Standard(vec![Identity::from_bytes(&recipient_buf)]),
        0x12 => Recipient::Flood(Identity::from_bytes(&recipient_buf)),
        code => panic!("Invalid recipient code: {}", code), // TODO: don't panic here
    };

    // Read 61 bytes
    let num = read_exactly(4, stream)?;
    let sig = read_exactly(16, stream)?;
    let seqid = read_exactly(32, stream)?;
    let next = read_exactly(9, stream)?;

    let seq = SeqData::from(unsafe {
        SeqDataEncoded {
            num: num.try_into().unwrap_unchecked(),
            sig: sig.try_into().unwrap_unchecked(),
            seqid: seqid.try_into().unwrap_unchecked(),
            next: next.try_into().unwrap_unchecked(),
        }
    });

    let payload_len_buf = read_exactly(4, stream)?;
    let payload_len = u32::from_be_bytes(unsafe { payload_len_buf.try_into().unwrap_unchecked() });
    let payload = read_exactly(payload_len as usize, stream)?;

    Ok(Frame {
        sender,
        recipient,
        seq,
        payload,
    })
}

#[test]
fn in_and_out() {
    let f = Frame::dummy();

    let mut buffer = vec![];
    encode_frame(&mut buffer, &f).unwrap();

    let f2 = decode_frame(&mut buffer.as_slice()).unwrap();
    assert_eq!(f, f2);
}
