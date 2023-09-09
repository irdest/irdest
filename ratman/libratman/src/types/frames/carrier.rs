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
#[derive(Debug)]
pub struct CarrierFrameMeta {
    pub inner: CarrierFrame,
    // FIXME: we only need a receive timestamp for queue sorting
    pub time: TimePair,
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

/// Carrier frame format
#[derive(Debug)]
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
    /// If this frame is in a series of frames that re-composes
    /// into a larger message, this identifier is used to
    /// re-assemble the original message stream.  If a frame is
    /// not part of a message sequence, this field MAP be NULL
    // TODO: 32-bytes for this might be totally overkill
    pub seq_id: Option<Id>,
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
        seq_id: Option<Id>,
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

    /// Calculate the size of the metadata head for this frame type
    fn get_meta_size(&self) -> u16 {
        let modes_size = core::mem::size_of_val(&self.modes);
        let recipient_size = match self.recipient {
            Some(_) => 32,
            None => 1,
        };
        let sender_size = core::mem::size_of_val(&self.sender);
        let seq_id_size = match self.seq_id {
            Some(_) => 32,
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
        let (input, seq_id) = parse::maybe_id(input)?;
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
        Some(Id::random()),
        None,
    );
    // 2 bytes of modes, 32 bytes each for the recipient, sender, and
    // seq_id and finally 1 zero-byte for the signature -> 99
    assert_eq!(f.get_meta_size(), 99);
}
