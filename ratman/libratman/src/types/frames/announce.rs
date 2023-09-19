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
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum AnnounceFrame {
    V1(AnnounceFrameV1),
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
    origin: OriginDataV1,
    /// Corresponding origin data signature
    origin_signature: Id,

    /// Mandatory route announcement data
    ///
    /// This field is not signed since any network participant between
    /// receiving the announcemend and re-broadcasting it MUST update
    /// the corresponding fields with data for the receiving channel.
    ///
    /// This means that the data can't be relied on to be
    /// cryptigraphically correct.
    route: RouteDataV1,
}

impl FrameGenerator for AnnounceFrameV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.origin.generate(buf)?;
        self.origin_signature.generate(buf)?;
        self.route.generate(buf)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct OriginDataV1 {
    timestamp: DateTime<Utc>,
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

impl FrameGenerator for RouteDataV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.mtu.generate(buf)?;
        self.size_hint.generate(buf)?;
        Ok(())
    }
}
