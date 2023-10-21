use crate::{
    client::Address,
    types::{
        frames::{generate, parse, FrameGenerator, FrameParser},
        Id, Recipient, SequenceIdV1,
    },
    EncodingError, Result,
};
use nom::IResult;

use super::modes;

//////
///////////   TOP LEVEL SECTION
///////////
/////////// Contains versioned structures and top-level encoding
/////////// utilities.  Sub-versions MUST NOT use custom encoding
/////////// facilities, to avoid duplication errors.

/// Contains top-level CarrierFrame metadata structure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CarrierFrameHeader {
    V1(CarrierFrameHeaderV1),
}

impl CarrierFrameHeader {
    /// Allocate a new Header for a netmod peering protocol
    pub fn new_netmodproto_frame(modes: u16, router_addr: Address, payload_length: u16) -> Self {
        Self::V1(CarrierFrameHeaderV1 {
            modes,
            recipient: None,
            sender: router_addr,
            seq_id: None,
            auxiliary_data: None,
            payload_length,
        })
    }

    /// Allocate a new header for an ERIS-block data frame
    pub fn new_blockdata_frame(
        sender: Address,
        recipient: Recipient,
        seq_id: SequenceIdV1,
        payload_length: u16,
    ) -> Self {
        Self::V1(CarrierFrameHeaderV1 {
            modes: modes::DATA,
            recipient: Some(recipient),
            sender,
            seq_id: Some(seq_id),
            auxiliary_data: None,
            payload_length,
        })
    }

    /// Allocate a new header for an address announcement frame
    pub fn new_announce_frame(sender: Address, payload_length: u16) -> Self {
        Self::V1(CarrierFrameHeaderV1 {
            modes: modes::ANNOUNCE,
            recipient: None,
            sender,
            seq_id: None,
            auxiliary_data: None,
            payload_length,
        })
    }

    pub fn get_blockdata_size(sender: Address, recipient: Recipient) -> usize {
        CarrierFrameHeader::new_blockdata_frame(
            sender,
            recipient,
            SequenceIdV1 {
                hash: Id::random(),
                num: 0,
                max: 0,
            },
            0,
        )
        .get_size()
    }

    /// Calculate the size of this metadata header
    pub fn get_size(&self) -> usize {
        match self {
            Self::V1(header) => {
                let modes_size = core::mem::size_of_val(&header.modes);
                let payload_len_size = core::mem::size_of_val(&header.payload_length);
                let sender_size = core::mem::size_of_val(&header.sender);
                let recipient_size = match header.recipient {
                    // Recipient adds one more byte to distinguish between
                    // Targeted and Flood send
                    Some(_) => 32 + 1,
                    None => 1,
                };
                let seq_id_size = match header.seq_id {
                    Some(ref seq_id) => core::mem::size_of_val(seq_id),
                    None => 1,
                };
                let aux_data_size = match header.auxiliary_data {
                    Some(_) => 32,
                    None => 1,
                };

                (1 // Include 1 byte for the version field itself
                    + modes_size
                    + sender_size
                    + recipient_size
                    + seq_id_size
                    + aux_data_size
                    + payload_len_size)
            }
        }
    }

    pub fn get_modes(&self) -> u16 {
        match self {
            Self::V1(inner) => inner.modes,
        }
    }

    pub fn get_sender(&self) -> Address {
        match self {
            Self::V1(inner) => inner.sender,
        }
    }

    pub fn get_recipient(&self) -> Option<Recipient> {
        match self {
            Self::V1(inner) => inner.recipient,
        }
    }

    pub fn get_seq_id(&self) -> Option<SequenceIdV1> {
        match self {
            Self::V1(inner) => inner.seq_id,
        }
    }

    pub fn get_payload_length(&self) -> usize {
        match self {
            Self::V1(inner) => inner.payload_length as usize,
        }
    }
}

impl FrameParser for CarrierFrameHeader {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take(1 as usize)(input)?;

        match version[0] {
            1 => {
                let (input, modes) = parse::take_u16(input)?;
                let (input, sender) = parse::take_address(input)?;
                let (input, recipient) = Option::<Recipient>::parse(input)?;
                let (input, seq_id) = SequenceIdV1::parse(input)?;
                let (input, auxiliary_data_slice) = parse::take(64 as usize)(input)?;
                let (input, payload_length) = parse::take_u16(input)?;

                let mut auxiliary_data = [0; 64];
                auxiliary_data.copy_from_slice(auxiliary_data_slice);

                Ok((
                    input,
                    Ok(CarrierFrameHeader::V1(CarrierFrameHeaderV1 {
                        modes,
                        sender,
                        recipient,
                        seq_id,
                        auxiliary_data: Some(auxiliary_data),
                        payload_length,
                    })),
                ))
            }
            unknown_version => Ok((
                input,
                Err(EncodingError::InvalidVersion(unknown_version).into()),
            )),
        }
    }
}

impl FrameGenerator for CarrierFrameHeader {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::V1(inner) => {
                buf.push(1); // version byte
                inner.modes.generate(buf)?;
                inner.sender.generate(buf)?;
                inner.recipient.generate(buf)?;
                inner.seq_id.generate(buf)?;
                inner.auxiliary_data.generate(buf)?;
                inner.payload_length.generate(buf)?;
            }
        }

        Ok(())
    }
}

//////
//////   VERSION 1 (2023)
//////
////// Introduce the basic version of the CarrierFrame metadata
////// header.  Most of the fields are optional, or very small.  The
////// only _mandatory_ data is the sender Address, without which
////// nothing else can happen.
//////
////// Conceptually auxiliary_data can be used for signatures (a
////// x25519-dalek signature is 64 bytes long), but since most
////// messages don't have to be signed, it can be re-used as a
////// timestamp indicator.
//////
////// Future versions may enforce the signature, and so timestamps
////// shouldn't be required for most messages.

/// Inner CarrierFrame metadata header (initial version)
///
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CarrierFrameHeaderV1 {
    /// Indicate the frame type enclosed by this carrier
    modes: u16,
    /// Mandatory sender address key
    ///
    /// For certain protocol messages this MAY be the router's root
    /// key.
    sender: Address,
    /// Optional recipient address key
    ///
    /// The recipient field MAY be NULL if the contained frame is
    /// addressed to the whole network, and not part of a flood
    /// namespace.  Only a limited number of frame types may set
    /// this condition (for example protocol announcements).
    recipient: Option<Recipient>,
    /// Optional sequence ID
    ///
    /// Any message that is too large to fit into a single carrier
    /// frame will need to be sliced across multiple carriers.  For
    /// each frame in the sequence, the same sequence ID hash MUST be
    /// used.  Additionally this field contains a numeric counter that
    /// can be used to re-order payloads on the recipient side, if
    /// they have arrived out of order.
    ///
    /// This field is not cryptographicly validated, and as such the
    /// payload encoding MUST be verified to ensure data integrity.
    seq_id: Option<SequenceIdV1>,
    /// Optional auxiliary data field
    ///
    /// Some message types may use this field for signatures, others
    /// for additional connection metadata.  MAY be left blank with a
    /// single zero-byte.
    auxiliary_data: Option<[u8; 64]>,
    /// Length of the trailing payload section
    payload_length: u16,
}
