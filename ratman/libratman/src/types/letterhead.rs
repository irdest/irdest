use crate::{
    frame::{generate::generate_cstring_tuple_vec, parse, FrameGenerator, FrameParser},
    types::{Address, Recipient},
    Result,
};
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::ffi::CString;

/// Message stream letterhead
///
/// This type is used by the sending and receiving routers to negotiate sending/
/// receiving behaviour of the two clients that wish to communicate.  In case
/// the receiver client is offline this type is cached in the receiving router's
/// metadata db to notify the client of an available stream when it comes back.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct LetterheadV1 {
    /// Who is sending this stream
    pub from: Address,
    /// Who should be receiving this stream
    pub to: Recipient,
    /// How much data is contained in the stream
    ///
    /// Any amount that fits into memory of the client application can easily be
    /// read in one go (via the `Chunk<T>` api), while larger stream sizes
    /// should most likely be read in multiple chunks to avoid the client
    /// running out of memory.
    pub payload_length: u64,
    /// Optional set of key-value attributes
    ///
    /// These can radically change the sending behaviour of your
    /// provided payload.  Please consult the manual for details.
    // todo: limit the auxiliary data size to allow it to be copy?
    pub auxiliary_data: Vec<(CString, CString)>,
}

impl FrameGenerator for LetterheadV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // version number -- currently not checked during decoding, but we will
        // add a wrapper type if we ever update this format
        buf.push(1);

        self.from.generate(buf)?;
        Some(self.to).generate(buf)?;
        self.payload_length.generate(buf)?;
        generate_cstring_tuple_vec(self.auxiliary_data, buf)?;
        Ok(())
    }
}

impl FrameParser for LetterheadV1 {
    type Output = Result<LetterheadV1>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, from) = parse::take_address(input)?;
        let (input, to) = Option::<Recipient>::parse(input)?;
        let (input, payload_length) = parse::take_u64(input)?;
        let (input, auxiliary_data) = parse::take_cstring_tuple_vec(input)?;

        Ok((
            input,
            auxiliary_data.map(|auxiliary_data| Self {
                from,
                to: to.unwrap(),
                payload_length,
                auxiliary_data,
            }),
        ))
    }
}
