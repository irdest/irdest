use crate::{
    frame::{carrier::CarrierFrameHeader, FrameGenerator, FrameParser},
    EncodingError, Result,
};

/// Container for carrier frame metadata and a full message buffer
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct InMemoryEnvelope {
    pub header: CarrierFrameHeader,
    /// WARNING: this buffer contains both the header and payload.  If you try
    /// to parse this buffer as a payload it will fail!  Use
    /// `get_payload_slice()` on InMemoryEnvelope instead, or if a function only
    /// has access to header and buffer separately you can do:
    /// `buffer.as_slice[header.get_size()..]`
    pub buffer: Vec<u8>,
}

impl InMemoryEnvelope {
    #[doc(hidden)]
    pub fn test_envelope() -> Self {
        let header = CarrierFrameHeader::new_announce_frame(crate::types::Address::random(), 0);
        let mut buffer = vec![];
        let _ = header.generate(&mut buffer);
        Self { header, buffer }
    }

    pub fn from_header_and_payload(
        header: CarrierFrameHeader,
        mut payload: Vec<u8>,
    ) -> Result<Self> {
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

    /// Get mutable access to the underlying payload section
    pub fn mut_payload_slice(&mut self) -> &mut [u8] {
        let header_end = self.header.get_size();
        &mut self.buffer.as_mut_slice()[header_end..]
    }
}
