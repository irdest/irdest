use crate::{
    frame::{parse, FrameGenerator, FrameParser},
    EncodingError, Result,
};
use chrono::{DateTime, Utc};
use nom::IResult;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum AnnounceFrame {
    V1(AnnounceFrameV1),
}

impl FrameParser for AnnounceFrame {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, v) = parse::take_byte(input)?;

        match v {
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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct AnnounceFrameV1 {
    /// Mandatory origin announce data
    ///
    /// This field, combined with the signature is used to verify that
    /// an announcement originated from the real address key.
    pub origin: OriginDataV1,
    /// Corresponding origin data signature
    pub origin_signature: [u8; 64],

    /// Mandatory route announcement data
    ///
    /// This field is not signed since the field must be updated after every
    /// re-dispatch with valuesfor the new receiving channel.
    ///
    /// Thus the data can't be relied on to be cryptigraphically correct.
    pub route: RouteDataV1,
}

impl FrameParser for AnnounceFrameV1 {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, origin) = OriginDataV1::parse(input)?;
        let (input, origin_signature) = parse::take_signature(input)?;
        let (input, route) = RouteDataV1::parse(input)?;

        Ok((
            input,
            origin.map(|origin| Self {
                origin,
                origin_signature,
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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PeerDataV1 {
    // Struct left blank as for the current version of the
    // specification
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct RouteDataV1 {
    /// Currently lowest MTU encountered by this announcement
    pub available_mtu: u32,
    /// Currently lowest available bandwidth encountered by this announcement
    pub available_bw: u32,
}

impl FrameParser for RouteDataV1 {
    type Output = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, available_mtu) = parse::take_u32(input)?;
        let (input, available_bw) = parse::take_u32(input)?;
        Ok((
            input,
            Self {
                available_mtu,
                available_bw,
            },
        ))
    }
}

impl FrameGenerator for RouteDataV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.available_mtu.generate(buf)?;
        self.available_bw.generate(buf)?;
        Ok(())
    }
}

#[test]
fn generate_parse_announce() {
    let origin = OriginDataV1::now();
    let origin_signature = [0; 64];

    // Create a full announcement and encode it
    let a = AnnounceFrame::V1(AnnounceFrameV1 {
        origin,
        origin_signature,
        route: RouteDataV1 {
            available_mtu: 0,
            available_bw: 0,
        },
    });

    let mut a_buf = vec![];
    a.clone().generate(&mut a_buf).unwrap();
    println!("Announce buf: {a_buf:?}");

    let (rem, a_dec) = AnnounceFrame::parse(a_buf.as_slice()).unwrap();
    println!("Remaining: {rem:?}");
    println!("Announce: {a_dec:?}");
    assert_eq!(rem.len(), 0);
    assert_eq!(a, a_dec.unwrap());
}
