// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! API encoding types for Ratman

mod api_util;
mod envelope;
mod identifiers;
mod letterhead;
mod platform;
mod recipient;
mod router;
mod sequence_id;
mod status;

pub mod error;

pub use api_util::*;
pub use envelope::InMemoryEnvelope;
pub use identifiers::{
    address::{Address, Namespace},
    id::Ident32,
    subnet::Subnet,
    target::Neighbour,
    ID_LEN,
};
pub use letterhead::LetterheadV1;
pub use platform::{Os, StateDirectoryLock};
pub use recipient::Recipient;
pub use router::RouterMeta;
pub use sequence_id::SequenceIdV1;
pub use status::CurrentStatus;

use std::ffi::CString;

/// A memcopy-able string shorter than 64 characters
///
/// It will explode in your face if you try to put bigger things into it :)
///
/// Unfortunately this type can't be encoded with Serde.  Use the
/// `FrameGenerator` and `FrameParser` traits instead!  An adapter
/// `SerdeFrametype` exists to make this integration seamless.
///
/// Can also be allocated into a CString for protocol encodings
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortStr([char; 64]);

impl ShortStr {
    pub fn from_str(s: &'static str) -> Self {
        assert!(s.len() < 64);
        ShortStr(
            s.chars()
                .enumerate()
                .fold(['\0'; 64], |mut buf, (counter, glyph)| {
                    buf[counter] = glyph;
                    buf
                }),
        )
    }

    pub fn into_cstring(self) -> CString {
        CString::from_vec_with_nul(self.0[..].iter().map(|x| *x as u8).collect::<Vec<_>>())
            .expect("failed to encode ShortStr to CString")
    }
}
