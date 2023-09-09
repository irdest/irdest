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

#[derive(Debug)]
pub struct OriginDataV1 {
    timestamp: DateTime<Utc>,
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
