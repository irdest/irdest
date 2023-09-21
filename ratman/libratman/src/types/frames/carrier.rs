use super::{
    generate::FrameGenerator,
    parse::{self, FrameParser, IResult},
};
use crate::{
    client::TimePair,
    types::{error::EncodingError, Address, Id, NonfatalError},
    RatmanError, Result,
};
use byteorder::{BigEndian, ByteOrder};

/// Never sent across the network, but carries additional metadata
// TODO: do we really need the full carrier frame?
#[derive(Debug)]
pub struct FullCarrierFrameMeta {
    pub inner: CarrierFrame,
    // FIXME: we only need a receive timestamp for queue sorting
    pub time: TimePair,
}

/// A partially parsed CarrierFrame
///
/// A data stream originates in the Netmod API, uses
/// `ProtoCarrierFrameMeta::from_peek(...)` to construct this metadata
/// header, which can then be used for protocol parsing.  Messages
/// that aren't directly addressed to this node, and are not required
/// for routing negotiation can be left un-parsed and forwarded to the
/// next transport hop as-is.
#[derive(Debug, Clone)]
pub struct ProtoCarrierFrameMeta {
    pub version: u8,
    pub modes: u16,
    pub recipient: Option<Address>,
    pub sender: Address,
}

use nom::{bytes::complete::take, combinator::peek};

impl ProtoCarrierFrameMeta {
    /// Peek into a data stream to read the first few meta fields
    pub fn from_peek(input: &[u8]) -> Result<Self> {
        match parse::peek_carrier_meta(input) {
            Ok((_, meta)) => Ok(meta),
            Err(_) => Err(EncodingError::Parsing(
                "Not enough data for a carrier frame metadata section!".to_owned(),
            )
            .into()),
        }
    }
}

/// Basic carrier frame format
///
/// All Irdest frame formats are versioned, meaning that different
/// versions can be parsed in a backwards compatible manner.
#[derive(Debug)]
pub enum CarrierFrame {
    V1(CarrierFrameV1),
}

impl CarrierFrame {
    pub fn get_max_payload(&self, mtu: u16) -> Result<u16> {
        let max_inner = match self {
            Self::V1(v1) => v1.get_max_size(mtu),
        };

        Ok(0)
    }
}

impl FrameParser for CarrierFrame {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take(1 as usize)(input)?;

        match version[0] {
            1 => {
                let (input, inner) = CarrierFrameV1::parse(input)?;
                Ok((input, Ok(CarrierFrame::V1(inner))))
            }
            unknown_version => Ok((
                input,
                Err(EncodingError::InvalidVersion(unknown_version).into()),
            )),
        }
    }
}

impl FrameGenerator for CarrierFrame {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::V1(inner) => {
                buf.push(1);
                inner.generate(buf)
            }
        }
    }
}

/// Block hash and a sequential counter to allow for re-ordering
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceIdV1 {
    pub hash: Id,
    pub num: u8,
}

impl FrameParser for SequenceIdV1 {
    type Output = Option<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, hash) = parse::maybe_id(input)?;
        match hash {
            Some(hash) => {
                let (input, num) = parse::take_byte(input)?;
                Ok((input, Some(Self { hash, num })))
            }
            None => Ok((input, None)),
        }
    }
}

impl FrameGenerator for SequenceIdV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.hash.generate(buf)?;
        buf.push(self.num);
        Ok(())
    }
}

/// Carrier frame format
#[derive(Clone, Debug, PartialEq)]
pub struct CarrierFrameV1 {
    /// Indicate the frame type enclosed by this carrier
    pub modes: u16,
    /// Optional recipient address key
    ///
    /// The recipient field MAY be NULL if the contained frame is
    /// addressed to the whole network, and not part of a flood
    /// namespace.  Only a limited number of frame types may set
    /// this condition (for example protocol announcements).
    pub recipient: Option<Address>,
    /// Mandatory sender address key
    pub sender: Address,
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
    pub seq_id: Option<SequenceIdV1>,
    /// Optional signature field
    ///
    /// Some frame payloads have internal signature handling.  In
    /// these cases this field MAY be ommitted.
    pub signature: Option<Id>,
    /// Frame payload field
    ///
    /// When encoding, the payload MUST be preceded with a u16
    /// length indicator.
    pub payload: Vec<u8>,
}

impl CarrierFrameV1 {
    pub fn pre_alloc(
        modes: u16,
        recipient: Option<Address>,
        sender: Address,
        seq_id: Option<SequenceIdV1>,
        signature: Option<Id>,
    ) -> Self {
        Self {
            modes,
            recipient,
            sender,
            seq_id,
            signature,
            payload: vec![],
        }
    }

    /// A utility function to fill in the payload field for a frame
    ///
    /// It first checks what the maximum size of the payload for the
    /// given MTU is allowed to be, and fails if it can't
    /// theoretically fit the metadata section.  Then it checks if
    /// there's enough space available for the desired data chunk.
    pub fn set_payload_checked(&mut self, mtu: u16, payload: Vec<u8>) -> Result<()> {
        if payload.len() > self.get_max_size(mtu)? as usize {
            // TODO: is this the right error type for this scenario?
            return Err(EncodingError::FrameTooLarge(payload.len()).into());
        }

        self.payload = payload;
        Ok(())
    }

    /// Calculate the size of the metadata head for this frame type
    fn get_meta_size(&self) -> u16 {
        let modes_size = core::mem::size_of_val(&self.modes);
        let recipient_size = match self.recipient {
            Some(_) => 32,
            None => 1,
        };
        let sender_size = core::mem::size_of_val(&self.sender);
        let seq_id_size = match self.seq_id {
            Some(ref seq_id) => core::mem::size_of_val(seq_id),
            None => 1,
        };
        let signature_size = match self.signature {
            Some(_) => 32,
            None => 1,
        };

        (modes_size + recipient_size + sender_size + seq_id_size + signature_size) as u16
    }

    /// Calculate the maximum payload size for a given MTU
    pub fn get_max_size(&self, mtu: u16) -> Result<u16> {
        let header_size = self.get_meta_size();
        let size: i32 = mtu as i32 - header_size as i32;

        if size > 0 && size < core::u16::MAX as i32 {
            Ok(size as u16)
        } else {
            Err(NonfatalError::MtuTooSmallForFrame.into())
        }
    }
}

impl FrameParser for CarrierFrameV1 {
    type Output = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, modes) = parse::take_u16(input)?;
        let (input, recipient) = parse::maybe_address(input)?;
        let (input, sender) = parse::take_address(input)?;
        let (input, seq_id) = SequenceIdV1::parse(input)?;
        let (input, signature) = parse::maybe_id(input)?;
        let (input, payload_length) = parse::take_u16(input)?;
        let (input, payload) = parse::take(payload_length as usize)(input)?;

        Ok((
            input,
            Self {
                modes,
                recipient,
                sender,
                seq_id,
                signature,
                // FIXME: can we do without copying the contents ?
                payload: payload.to_vec(),
            },
        ))
    }
}

impl FrameGenerator for CarrierFrameV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.modes.generate(buf)?;
        self.recipient.generate(buf)?;
        self.sender.generate(buf)?;
        self.seq_id.generate(buf)?;
        self.signature.generate(buf)?;
        self.payload.generate(buf)?;
        Ok(())
    }
}

impl From<CarrierFrameV1> for CarrierFrame {
    fn from(inner: CarrierFrameV1) -> Self {
        Self::V1(inner)
    }
}

#[test]
#[allow(deprecated)]
fn v1_announce_frame_meta_size() {
    let f = CarrierFrameV1::pre_alloc(0, None, Address::random(), None, None);
    // 2 bytes of modes, 1 zero-byte for the recipient, 32 bytes for
    // the sender, and each 1 zero-byte for seq_id and signature -> 37
    assert_eq!(f.get_meta_size(), 37);
}

#[test]
#[allow(deprecated)]
fn v1_data_frame_meta_size() {
    let f = CarrierFrameV1::pre_alloc(
        2,
        Some(Address::random()),
        Address::random(),
        Some(SequenceIdV1 {
            hash: Id::random(),
            num: 123,
        }),
        None,
    );
    // 2 bytes of modes, 32 bytes each for the recipient, sender, a
    // 32-byte seq_id and 1-byte re-ordering counter and finally 1
    // zero-byte for the signature -> 100
    assert_eq!(f.get_meta_size(), 100);
}

/// Ensure that encoding and decoding is possible from and to the same
/// binary representation.  This function constructs a fake carrier
/// frame which is not used in normally, but which tests every field
#[test]
#[allow(deprecated)]
fn v1_empty_carrier() {
    let mut f = CarrierFrameV1::pre_alloc(
        1312,
        Some(Address::random()),
        Address::random(),
        Some(SequenceIdV1 {
            hash: Id::random(),
            num: 123,
        }),
        Some(Id::random()),
    );
    f.set_payload_checked(1300, super::random_payload(1024));

    let mut encoded = vec![];
    f.clone().generate(&mut encoded).unwrap();

    let (_, decoded) = CarrierFrameV1::parse(&encoded).unwrap();

    assert_eq!(f, decoded);
}
