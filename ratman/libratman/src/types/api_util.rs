use nom::{combinator::peek, IResult};
use serde::{Deserialize, Serialize};

use crate::{
    frame::{
        parse::{self as fparse, take_byte},
        FrameGenerator, FrameParser,
    },
    types::Ident32,
    Result,
};
use std::{
    collections::BTreeMap,
    ffi::CString,
    fmt::{self, Debug, Formatter},
};

pub fn to_cstring(s: &String) -> CString {
    CString::new(s.as_bytes()).expect("String could not be turned into CString")
}

/// A simple authentication object for an address
///
/// A client can have multiple AddrAuth objects active at once, one for each
/// address that it is using.  An auth object is required to have access to
/// address functions and is used to encrypt the address private key on disk.
///
/// **It is thus strongly recommended to store this auth token encrypted with a
/// user passphrase**
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AddrAuth {
    pub token: Ident32,
}

impl AddrAuth {
    pub fn new() -> Self {
        Self {
            token: Ident32::random(),
        }
    }
}

impl Debug for AddrAuth {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ClientAuth {{ token: _ }}")
    }
}

impl FrameGenerator for AddrAuth {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.token.generate(buf)?;
        Ok(())
    }
}

impl FrameGenerator for Option<AddrAuth> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(auth) => auth.generate(buf)?,
            None => buf.push(0),
        }
        Ok(())
    }
}

impl FrameParser for AddrAuth {
    type Output = Option<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, exists) = peek(take_byte)(input)?;

        if exists == 0 {
            // take the byte after all
            let (input, _) = take_byte(input)?;
            Ok((input, None))
        } else {
            let (input, token) = fparse::maybe_id(input)?;

            match token {
                Some(token) => Ok((input, Some(Self { token }))),
                None => Ok((input, None)),
            }
        }
    }
}

/// Apply a tri-state modification to an existing Option<T>
pub enum Modify<T> {
    Keep,
    Change(T),
    DeleteOne(T),
    DeleteAll,
}

/// Apply a Modify object to an Option
pub fn apply_simple_modify<T>(base: &mut Option<T>, mobj: Modify<T>) {
    match mobj {
        // Don't apply a change
        Modify::Keep => {}
        // We would love to do something
        // here but we don't know how to since T isn't a Map
        Modify::DeleteOne(_) => {
            warn!(
                "Function `apply_simple_modify` called with an invalid operand: \
                   Modify::DeleteOne(_) is not implemented by this function."
            );
        }
        // Delete value
        Modify::DeleteAll => *base = None,
        // Add value
        Modify::Change(new) => *base = Some(new),
    }
}

pub fn apply_map_modify<T: Ord>(base: &mut BTreeMap<T, T>, mobj: Modify<T>) {
    match mobj {
        Modify::DeleteOne(ref key) => {
            base.remove(key);
        }
        _ => {
            warn!("Function `apply_map_modify` called with a non-Map base operand");
        }
    };
}

/// Apply a simple filter for trust relationships
pub enum TrustFilter {
    GreatEq(u8),
    Less(u8),
}
