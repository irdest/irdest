//! Networking frames

use crate::types::{
    frames::{CarrierFrameHeader, FrameGenerator, FrameParser},
    Address, EncodingError, Result,
};
use std::fmt::{self, Display};

/// Describes an endpoint's send target
///
/// This is different from a Recipient in that it doesn't encode
/// information about a user on the global network.  It's values are
/// used by one-to-many Endpoint implementors to disambiguate their
/// own routing tables to Ratman without needing to share actual
/// connection information.
///
/// If your endpoint doesn't implement a one-to-many link (i.e. if
/// it's always one-to-one), just let this value to `Single(0)`
/// (`Target::default()`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Send message to all reachable endpoints
    Flood(Address),
    /// Encodes a specific target ID
    Single(u16),
}

impl Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Flood(addr) => format!("Flood({})", addr),
                Self::Single(t) => format!("Peer({})", t),
            }
        )
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::Single(0)
    }
}

/// Container for carrier frame metadata and a full message buffer
#[derive(Debug, Clone)]
pub struct InMemoryEnvelope {
    pub header: CarrierFrameHeader,
    pub buffer: Vec<u8>,
}

impl InMemoryEnvelope {
    pub fn from_header(header: CarrierFrameHeader, mut payload: Vec<u8>) -> Result<Self> {
        let mut buffer = vec![];
        header.generate(&mut buffer)?;
        buffer.append(&mut payload);
        Ok(InMemoryEnvelope { header, buffer })
    }

    pub fn parse_from_buffer(buf: Vec<u8>) -> Result<Self> {
        let header = match CarrierFrameHeader::parse(buf.as_slice()) {
            Ok((_, Ok(h))) => h,
            Ok((_, Err(e))) => return Err(EncodingError::Parsing(e.to_string()).into()),
            Err(e) => return Err(EncodingError::Parsing(e.to_string()).into()),
        };

        Ok(InMemoryEnvelope {
            header,
            buffer: buf
                .into_iter()
                .take(header.get_size() + header.get_payload_length())
                .collect(),
        })
    }

    /// Get access to the buffer section representing the payload
    pub fn get_payload_slice(&self) -> &[u8] {
        let header_end = self.header.get_size();
        &self.buffer.as_slice()[header_end..]
    }
}
