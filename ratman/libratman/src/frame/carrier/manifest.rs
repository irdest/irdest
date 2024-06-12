use crate::{
    frame::{parse, FrameGenerator, FrameParser},
    types::{Ident32, LetterheadV1},
    BlockError, EncodingError, Result,
};
use async_eris::{BlockKey, BlockReference, ReadCapability};
use nom::{AsBytes, IResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
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
                Ok((input, inner.map(|inner| ManifestFrame::V1(inner))))
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
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ManifestFrameV1 {
    /// Full stream metadata
    pub letterhead: LetterheadV1,
    /// Block size for this manifest set
    pub block_size: u8,
    /// Block level indicator
    pub block_level: u8,
    /// Block root reference
    pub root_reference: Ident32,
    /// Root block key
    pub root_key: Ident32,
}

impl FrameParser for ManifestFrameV1 {
    type Output = Result<Self>;

    fn parse(input: &[u8]) -> IResult<&[u8], Result<Self>> {
        let (input, letterhead) = LetterheadV1::parse(input)?;
        let (input, block_size) = parse::take_byte(input)?;
        let (input, block_level) = parse::take_byte(input)?;
        let (input, root_reference) = parse::take_id(input)?;
        let (input, root_key) = parse::take_id(input)?;

        Ok((
            input,
            letterhead.map(|letterhead| Self {
                letterhead,
                block_size,
                block_level,
                root_reference,
                root_key,
            }),
        ))
    }
}

impl FrameGenerator for ManifestFrameV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.letterhead.generate(buf)?;
        buf.push(self.block_size);
        buf.push(self.block_level);
        buf.extend_from_slice(self.root_reference.as_bytes());
        buf.extend_from_slice(self.root_key.as_bytes());
        Ok(())
    }
}

impl From<(ReadCapability, LetterheadV1)> for ManifestFrameV1 {
    fn from((rc, lh): (ReadCapability, LetterheadV1)) -> Self {
        Self {
            letterhead: lh,
            block_size: match rc.block_size {
                // todo: support small non-standard frame sizes
                1024 => 1,
                32768 => 32,
                _ => unreachable!(),
            },
            block_level: rc.level,
            root_reference: Ident32::from_bytes(rc.root_reference.as_bytes()),
            root_key: Ident32::from_bytes(rc.root_key.as_bytes()),
        }
    }
}

impl From<ManifestFrameV1> for Result<ReadCapability> {
    fn from(mf: ManifestFrameV1) -> Self {
        Ok(ReadCapability {
            root_reference: BlockReference::try_from(&mf.root_reference.to_string())
                .map_err(|e| BlockError::Eris(e))?,
            root_key: BlockKey::try_from(&mf.root_key.to_string())
                .map_err(|e| BlockError::Eris(e))?,
            level: mf.block_level,
            block_size: match mf.block_size {
                1 => 1024,
                32 => 1024 * 32,
                _ => unreachable!(),
            },
        })
    }
}
