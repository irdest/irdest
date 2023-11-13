use super::{
    generate::FrameGenerator,
    parse::{self, FrameParser, IResult},
};
use crate::{
    types::{Address, Id, TimePair},
    EncodingError, NonfatalError, RatmanError, Result,
};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum AnnounceFrame {
    V1(AnnounceFrameV1),
}

impl FrameParser for AnnounceFrame {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, v) = parse::take(1 as usize)(input)?;

        match v[0] {
            1 => {
                let (input, inner) = AnnounceFrameV1::parse(input)?;
                Ok((input, inner.map(|inner| AnnounceFrame::V1(inner))))
            }
            unknown_version => Ok((
                input,
                Err(EncodingError::InvalidVersion(unknown_version).into()),
            )),
        }
    }
}

impl FrameGenerator for AnnounceFrame {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::V1(v1) => {
                buf.push(1); // Prepend the version
                v1.generate(buf)
            }
        }
    }
}

#[derive(Debug)]
pub struct AnnounceFrameV1 {
    /// Mandatory origin announce data
    ///
    /// This field, combined with the signature is used to verify that
    /// an announcement originated from the real address key.  Replay
    /// attacks are _somewhat_ possible, but since every router MUST
    /// keep track of announcement timestamps that have been seen
    /// before, only parts of the networks that didn't see the
    /// original announcement can be fooled by a maliciously crafted
    /// announcement frame.
    pub origin: OriginDataV1,
    /// Corresponding origin data signature
    pub origin_signature: [u8; 64],

    /// Mandatory route announcement data
    ///
    /// This field is not signed since any network participant between
    /// receiving the announcemend and re-broadcasting it MUST update
    /// the corresponding fields with data for the receiving channel.
    ///
    /// This means that the data can't be relied on to be
    /// cryptigraphically correct.
    pub route: RouteDataV1,
}

impl FrameParser for AnnounceFrameV1 {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, origin) = OriginDataV1::parse(input)?;
        let (input, origin_signature) = parse::maybe_signature(input)?;
        let (input, route) = RouteDataV1::parse(input)?;

        Ok((
            input,
            origin.map(|origin| Self {
                origin,
                // fixme: handle error explicitly
                origin_signature: origin_signature.unwrap(),
                route,
            }),
        ))
    }
}

impl FrameGenerator for AnnounceFrameV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.origin.generate(buf)?;
        self.origin_signature.generate(buf)?;
        self.route.generate(buf)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct OriginDataV1 {
    timestamp: DateTime<Utc>,
}

impl OriginDataV1 {
    /// Create an OriginDataV1 with the current time in Utc
    pub fn now() -> Self {
        Self {
            timestamp: Utc::now(),
        }
    }
}

impl FrameParser for OriginDataV1 {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, timestamp) = parse::take_datetime(input)?;
        Ok((input, timestamp.map(|timestamp| Self { timestamp })))
    }
}

impl FrameGenerator for OriginDataV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.timestamp.generate(buf)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PeerDataV1 {
    // Struct left blank as for the current version of the
    // specification
}

#[derive(Debug)]
pub struct RouteDataV1 {
    /// Currently lowest MTU encountered by this announcement
    pub mtu: u16,
    /// Currently lowest size_hint encountered by this announcement
    pub size_hint: u16,
}

impl FrameParser for RouteDataV1 {
    type Output = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, mtu) = parse::take_u16(input)?;
        let (input, size_hint) = parse::take_u16(input)?;
        Ok((input, Self { mtu, size_hint }))
    }
}

impl FrameGenerator for RouteDataV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.mtu.generate(buf)?;
        self.size_hint.generate(buf)?;
        Ok(())
    }
}
