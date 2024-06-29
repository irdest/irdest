use crate::{
    frame::{
        generate::generate_cstring_tuple_vec,
        parse::{self},
        FrameGenerator, FrameParser,
    },
    types::{Address, Recipient},
    Result,
};
use chrono::Utc;
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::ffi::CString;

use super::Ident32;

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
    pub stream_size: u64,
    /// Optional set of key-value attributes
    ///
    /// These can radically change the sending behaviour of your
    /// provided payload.  Please consult the manual for details.
    // todo: limit the auxiliary data size to allow it to be copy?
    pub auxiliary_data: Vec<(CString, CString)>,
}

impl LetterheadV1 {
    pub fn send(from: Address, to: Recipient) -> Self {
        Self {
            from,
            to,
            stream_size: 0,
            auxiliary_data: vec![],
        }
    }

    pub fn digest(&self) -> Ident32 {
        let mut plen_buf = vec![];
        self.stream_size.generate(&mut plen_buf).unwrap();

        let pl_len: &[u8] = plen_buf.as_slice();

        // let v: Vec<(String, String)> = self
        //     .auxiliary_data
        //     .iter()
        //     .map(|(k, v)| (k.into_string().unwrap(), v.into_string().unwrap()))
        //     .collect();

        let inner_addr = self.to.inner_address();
        let digest_input = vec![self.from.as_bytes(), inner_addr.as_bytes(), pl_len];

        // digest_input.append(&mut v.into_iter().fold(vec![], |mut vec, (k, v)| {
        //     vec.push(k.as_bytes());
        //     vec.push(v.as_bytes());
        //     vec
        // }));

        let mut vec = vec![];
        for buf in digest_input {
            buf.into_iter().for_each(|x| {
                vec.push(*x);
            });
        }

        Ident32::with_digest(&vec)
    }

    pub fn add_send_time(mut self) -> Self {
        self.auxiliary_data.push((
            CString::new("time-sent").unwrap(),
            CString::new(Utc::now().to_string()).unwrap(),
        ));
        self
    }

    /// Add your own metadata to the stream
    ///
    /// This data is only attached to the stream *Manifest* message and will be
    /// passed to the receiving client ahead of reading the incoming stream.
    ///
    /// **Note** this data is NOT being encrypted and every network participant
    /// will be able to see it.
    pub fn add_aux_data(mut self, key: impl Into<Vec<u8>>, val: impl Into<Vec<u8>>) -> Self {
        let key = CString::new(key).expect("invalid key data!");
        let val = CString::new(val).expect("invalid value data!");

        self.auxiliary_data.push((key, val));
        self
    }

    /// Turn a single letterhead into a set of letterheads to multiple recipients
    ///
    /// The `to`, `payload_length`, and `auxiliary_data` fields are copied from
    /// the initial letterhead, so any metadata you want to include in the
    /// auxiliary data section must be attached before calling this function.
    pub fn to_many(self, additional_recipients: Vec<Recipient>) -> Vec<Self> {
        let mut vec = vec![self.clone()];
        additional_recipients.into_iter().for_each(|recipient| {
            let mut new_lh = self.clone();
            new_lh.to = recipient;
            vec.push(new_lh)
        });
        vec
    }
}

impl FrameGenerator for LetterheadV1 {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        // version number -- currently not checked during decoding, but we will
        // add a wrapper type if we ever update this format
        buf.push(1);

        self.from.generate(buf)?;
        Some(self.to).generate(buf)?;
        self.stream_size.generate(buf)?;
        generate_cstring_tuple_vec(self.auxiliary_data, buf)?;
        Ok(())
    }
}

impl FrameParser for LetterheadV1 {
    type Output = Result<LetterheadV1>;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, lh_version) = parse::take_byte(input)?;
        assert!(lh_version == 1);
        let (input, from) = parse::take_address(input)?;
        let (input, to) = Option::<Recipient>::parse(input)?;
        let (input, payload_length) = parse::take_u64(input)?;
        let (input, auxiliary_data) = parse::take_cstring_tuple_vec(input)?;

        trace!("Parsed: {from:?} {to:?}, {payload_length:?}");

        Ok((
            input,
            auxiliary_data.map(|auxiliary_data| Self {
                from,
                to: to.unwrap(),
                stream_size: payload_length,
                auxiliary_data,
            }),
        ))
    }
}
