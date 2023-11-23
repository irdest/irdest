use crate::{frame::micro::parse::*, frame::FrameParser, types::TrustFilter, Result};
use nom::IResult;
use std::ffi::CString;

#[repr(C)]
pub struct PeerQuery {
    /// A regex filter that will have to pass on the peer note contents
    pub note_filter: Option<CString>,
    /// A filter for the existence of a tag for said peer
    pub tag_filter: Vec<CString>,
    /// A filter for a trust threashold for said peer
    pub trust_filter: Option<TrustFilter>,
}

impl FrameParser for PeerQuery {
    type Output = Result<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, note_filter) = maybe(cstring, input).map(|(i, x)| (i, x.flatten()))?;
        let (input, tag_filter) = vec_of(cstring, input)?;

        Ok((
            input,
            Ok(Self {
                note_filter,
                tag_filter: tag_filter.into_iter().filter_map(|x| x).collect(),
                trust_filter: None,
            }),
        ))
    }
}

#[repr(C)]
pub struct PeerList {
    pub interactive: bool,
}
