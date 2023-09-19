use super::{
    generate::FrameGenerator,
    parse::{self, FrameParser},
};
use crate::{
    types::{error::EncodingError, Id},
    Result,
};
use nom::IResult;

pub enum ManifestFrame {
    V1(ManifestFrameV1),
}

impl FrameParser for ManifestFrame {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, version) = parse::take_byte(input)?;
        match version {
            1 => {
                let (input, inner) = ManifestFrameV1::parse(input)?;
                Ok((input, Ok(ManifestFrame::V1(inner))))
            }
            unknown_version => Ok((
                input,
                Err(EncodingError::InvalidVersion(unknown_version).into()),
            )),
        }
    }
}

impl FrameGenerator for ManifestFrame {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::V1(v1) => {
                buf.push(1);
                v1.generate(buf)
            }
        }
    }
}

/// Encode the format of an ERIS manifest
///
/// This format follows the ERIS binary format specification [1]
///
/// [1]: https://eris.codeberg.page/spec/#name-binary-encoding-of-read-cap
pub struct ManifestFrameV1 {
    /// Block size for this manifest set
    block_size: u8,
    /// Block level indicator
    block_level: u8,
    /// Block root reference
    root_reference: Id,
    /// Root block key
    root_key: Id,
}

impl FrameParser for ManifestFrameV1 {
    type Output = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, block_size) = parse::take_byte(input)?;
        let (input, block_level) = parse::take_byte(input)?;
        let (input, root_reference) = parse::take_id(input)?;
        let (input, root_key) = parse::take_id(input)?;

        Ok((
            input,
            Self {
                block_size,
                block_level,
                root_reference,
                root_key,
            },
        ))
    }
}

impl FrameGenerator for ManifestFrameV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self.block_size);
        buf.push(self.block_level);
        buf.extend_from_slice(self.root_reference.as_bytes());
        buf.extend_from_slice(self.root_key.as_bytes());
        Ok(())
    }
}
