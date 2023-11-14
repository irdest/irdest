use crate::{
    frame::{carrier::parse, FrameGenerator, FrameParser},
    types::Id,
    Result,
};
use nom::IResult;
use serde::{Deserialize, Serialize};

/// Block hash and a sequential counter to allow carrier re-ordering
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceIdV1 {
    /// The block's content reference
    pub hash: Id,
    /// Number of THIS frame
    pub num: u8,
    /// Number of the LAST frame in the sequence
    pub max: u8,
}

impl FrameParser for SequenceIdV1 {
    type Output = Option<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, hash) = parse::maybe_id(input)?;
        match hash {
            Some(hash) => {
                let (input, num) = parse::take_byte(input)?;
                let (input, max) = parse::take_byte(input)?;
                Ok((input, Some(Self { hash, num, max })))
            }
            None => Ok((input, None)),
        }
    }
}

impl FrameGenerator for SequenceIdV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.hash.generate(buf)?;
        buf.push(self.num);
        buf.push(self.max);
        Ok(())
    }
}
