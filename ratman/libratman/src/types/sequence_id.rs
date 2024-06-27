use crate::{
    frame::{
        parse::{self, take_byte},
        FrameGenerator, FrameParser,
    },
    types::Ident32,
    Result,
};
use nom::{combinator::peek, IResult};
use serde::{Deserialize, Serialize};

/// Block hash and a sequential counter to allow carrier re-ordering
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceIdV1 {
    /// The block's content reference
    pub hash: Ident32,
    /// Number of THIS frame
    pub num: u8,
    /// Number of the LAST frame in the sequence
    pub max: u8,
}

impl FrameParser for SequenceIdV1 {
    type Output = Option<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, exists) = take_byte(input)?;

        if exists > 0 {
            let (input, hash) = parse::take_id(input)?;
            let (input, num) = parse::take_byte(input)?;
            let (input, max) = parse::take_byte(input)?;
            Ok((input, Some(Self { hash, num, max })))
        } else {
            let (input, _) = take_byte(input)?;
            Ok((input, None))
        }
    }
}

impl FrameGenerator for Option<SequenceIdV1> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(s) => {
                buf.push(1);
                s.hash.generate(buf)?;
                buf.push(s.num);
                buf.push(s.max);
            }
            None => buf.push(0),
        }
        Ok(())
    }
}

#[test]
fn test_encoding() {
    let num = rand::random::<u8>() % 7;

    let seq = dbg!(SequenceIdV1 {
        hash: Ident32::random(),
        num,
        max: num * 3,
    });

    let mut buf = vec![];
    Some(seq).generate(&mut buf).unwrap();

    let (rem, seq2) = SequenceIdV1::parse(buf.as_slice()).unwrap();

    let empty: &[u8] = &[];
    assert_eq!(rem, empty);
    assert_eq!(seq, seq2.unwrap());
}
