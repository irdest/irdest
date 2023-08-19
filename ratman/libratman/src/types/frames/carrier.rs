use crate::{
    client::TimePair,
    types::{frames::FrameEncoder, Address, Id},
    Result,
};
use byteorder::{BigEndian, ByteOrder};

/// Never sent across the network, but carries additional metadata
pub struct CarrierFrameMeta {
    pub inner: CarrierFrame,
    pub time: TimePair,
}

/// Basic carrier frame format
///
/// All Irdest frame formats are versioned, meaning that different
/// versions can be parsed in a backwards compatible manner.
#[repr(C)]
#[derive(Clone, Debug)]
pub enum CarrierFrame {
    V1 {
        /// Indicate the frame type enclosed by this carrier
        modes: [u8; 2],
        /// Optional recipient address key
        ///
        /// The recipient field MAY be NULL if the contained frame is
        /// addressed to the whole network, and not part of a flood
        /// namespace.  Only a limited number of frame types may set
        /// this condition (for example protocol announcements).
        recipient: Option<[u8; 32]>,
        /// Mandatory sender address key
        sender: [u8; 32],
        /// Optional sequence ID
        ///
        /// If this frame is in a series of frames that re-composes
        /// into a larger message, this identifier is used to
        /// re-assemble the original message stream.  If a frame is
        /// not part of a message sequence, this field MAP be NULL
        seq_id: Option<[u8; 32]>,
        /// Optional signature field
        ///
        /// Some frame payloads have internal signature handling.  In
        /// these cases this field MAY be ommitted.
        signature: Option<[u8; 32]>,
        /// Frame payload field
        ///
        /// When encoding, the payload MUST be preceded with a u16
        /// length indicator.
        payload: Vec<u8>,
    },
}

impl CarrierFrame {
    /// Pre-allocate a basic frame structure
    pub fn pre_alloc(
        modes: u16,
        recipient: Option<Address>,
        sender: Address,
        seq_id: Option<Id>,
        signature: Option<Id>,
    ) -> Self {
        Self::V1 {
            modes: super::u16_to_big_endian(modes),
            recipient: recipient.map(|r| r.slice()),
            sender: sender.slice(),
            seq_id: seq_id.map(|s| s.slice()),
            signature: signature.map(|s| s.slice()),
            payload: vec![],
        }
    }

    pub fn fill_payload(&mut self, f: &[u8]) {
        match self {
            Self::V1 {
                ref mut payload, ..
            } => {
                *payload = f.to_vec();
            }
        }
    }

    fn get_meta_size(&self) -> u16 {
        match self {
            Self::V1 {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                ..
            } => {
                let modes_size = core::mem::size_of_val(modes);
                let recipient_size = match recipient {
                    Some(_) => 32,
                    None => 1,
                };
                let sender_size = core::mem::size_of_val(sender);
                let seq_id_size = match seq_id {
                    Some(_) => 32,
                    None => 1,
                };
                let signature_size = match signature {
                    Some(_) => 32,
                    None => 1,
                };

                (modes_size + recipient_size + sender_size + seq_id_size + signature_size) as u16
            }
        }
    }

    /// Return the number of bytes available for the payload
    ///
    /// This function is relevant for any frame type that chooses to
    /// fill the entire available payload space (for data transfer).
    /// Calling this function MAY be ommited for small protocol frames
    /// (for example announcements), that can't be split across
    /// multiple frames anyway.
    pub fn get_max_size(&self, mtu: u16) -> u16 {
        match self {
            Self::V1 {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                ..
            } => {
                let meta_size = self.get_meta_size();
                let size: i32 = (mtu - meta_size) as i32;

                // TODO: replace this with a runtime error
                assert!(size > 0 && size < core::u16::MAX as i32);
                size as u16
            }
        }
    }
}

impl FrameEncoder for CarrierFrame {
    fn encode(mut self) -> Vec<u8> {
        let mut buf = vec![];

        match self {
            Self::V1 {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                mut payload,
            } => {
                // Include a version byte
                buf.push(1 as u8);

                // Encode our modes as two bytes
                buf.append(&mut modes.to_vec());

                // Append 32 bytes for the recipient, or a single NULL
                // byte to indicate that it's missing.
                buf.append(&mut match recipient {
                    Some(r) => r.to_vec(),
                    None => [0].to_vec(),
                });

                // Encode the sender address
                buf.append(&mut sender.to_vec());

                // Append 32 bytes for the seq_id, or a single NULL
                // byte to indicate that it's missing.
                buf.append(&mut match seq_id {
                    Some(id) => id.to_vec(),
                    None => [0].to_vec(),
                });

                // Append 32 bytes for the signature, or a single NULL
                // byte to indicate that it's missing.
                buf.append(&mut match signature {
                    Some(sig) => sig.to_vec(),
                    None => [0].to_vec(),
                });

                // Append the payload length as a u16
                buf.append(
                    &mut super::u16_to_big_endian(
                        payload
                            .len()
                            .try_into()
                            .expect("payload? more like faiload haha"),
                    )
                    .to_vec(),
                );

                // Finally append the payload
                buf.append(&mut payload);
            }
        }

        buf
    }

    // TODO: make this function move the data instead of copying
    // FIXME: Make this function not panic every 5 seconds
    fn decode(buf: &Vec<u8>) -> Result<Self> {
        let mut read_ctr: usize = 0;

        // First byte is the version
        let version = buf[read_ctr];
        read_ctr += 1;

        // Ensure the version is correct
        if version != 1 {
            return Err(crate::RatmanError::DesequenceFault);
        }

        // Next two bytes are the modes bitfield
        let modes = &buf[read_ctr..read_ctr + 2];
        read_ctr += 2;

        // Read one byte to see if a recipient is present
        let is_recipient = buf[read_ctr];
        let mut recipient = None;
        if is_recipient != 0 {
            // If it is, we reread (without incrementing) 32 bytes
            recipient = Some(&buf[read_ctr..read_ctr + 32]);
            read_ctr += 32;
        } else {
            // If it doesn't we increment the read_ctr for our zero byte
            read_ctr += 1;
        }

        // Then read the mandatory 32 byte sender
        let sender = &buf[read_ctr..read_ctr + 32];
        read_ctr += 32;

        // Read one byte to see if a seq_id is present
        let is_seq_id = buf[read_ctr];
        let mut seq_id = None;
        if is_seq_id != 0 {
            // If it is, we reread (without incrementing) 32 bytes
            seq_id = Some(&buf[read_ctr..read_ctr + 32]);
            read_ctr += 32;
        } else {
            // If it doesn't we increment the read_ctr for our zero byte
            read_ctr += 1;
        }

        // Read one byte to see if a signature is present
        let is_signature = buf[read_ctr];
        let mut signature = None;
        if is_signature != 0 {
            // If it is, we reread (without incrementing) 32 bytes
            signature = Some(&buf[read_ctr..read_ctr + 32]);
            read_ctr += 32;
        } else {
            // If it doesn't we increment the read_ctr for our zero byte
            read_ctr += 1;
        }

        // Read the mandatory 2 byte payload size
        let pl_size = BigEndian::read_u16(&buf[read_ctr..read_ctr + 2]);
        read_ctr += 2;

        // Then read pl_size many bytes for the payload
        let payload = &buf[read_ctr..read_ctr + pl_size as usize];
        read_ctr += pl_size as usize;

        // Make sure nobody is trying anything funny by dangling data off the end
        assert_eq!(read_ctr, buf.len());

        // Now put it all together into a beautiful CarrierFrame
        Ok(Self::V1 {
            modes: [modes[0], modes[1]],
            recipient: recipient.map(|r| Address::from_bytes(r).slice()),
            sender: Address::from_bytes(sender).slice(),
            seq_id: seq_id.map(|s| Id::from_bytes(s).slice()),
            signature: signature.map(|s| Id::from_bytes(s).slice()),
            payload: payload.to_vec(),
        })
    }
}

#[test]
#[allow(deprecated)]
fn announce_frame_meta_size() {
    let f = CarrierFrame::pre_alloc(0, None, Address::random(), None, None);
    // 2 bytes of modes, 1 zero-byte for the recipient, 32 bytes for
    // the sender, and each 1 zero-byte for seq_id and signature -> 37
    assert_eq!(f.get_meta_size(), 37);
}

#[test]
#[allow(deprecated)]
fn data_frame_meta_size() {
    let f = CarrierFrame::pre_alloc(
        2,
        Some(Address::random()),
        Address::random(),
        Some(Id::random()),
        None,
    );
    // 2 bytes of modes, 32 bytes each for the recipient, sender, and
    // seq_id and finally 1 zero-byte for the signature -> 99
    assert_eq!(f.get_meta_size(), 99);
}

#[test]
#[allow(deprecated)]
fn empty_frame_encode_decode() {
    let mut f = CarrierFrame::pre_alloc(2, None, Address::random(), None, None);
    let my_payload = "Hello, this is a very simple String payload".to_owned();
    f.fill_payload(my_payload.as_bytes());

    let encoded = f.clone().encode();

    println!("{:?}", encoded);

    // Persist the output so we can look at it in a hex viewer
    if let Some(mut f) = std::fs::File::create("./carrier.bin").ok() {
        use std::io::Write;
        f.write_all(&encoded).unwrap();
    }

    let decoded = CarrierFrame::decode(&encoded).unwrap();

    match (f, decoded) {
        (
            CarrierFrame::V1 {
                modes: f_modes,
                recipient: f_recipient,
                sender: f_sender,
                seq_id: f_seq_id,
                signature: f_signature,
                payload: f_payload,
            },
            CarrierFrame::V1 {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                payload,
            },
        ) => {
            assert_eq!(modes, f_modes);
            assert_eq!(recipient, f_recipient);
            assert_eq!(sender, f_sender);
            assert_eq!(seq_id, f_seq_id);
            assert_eq!(signature, f_signature);
            assert_eq!(payload, f_payload);

            assert_eq!(payload.as_slice(), my_payload.as_bytes());
        }
    }
}
