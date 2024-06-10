use nom::IResult;
use serde::{Deserialize, Serialize};

use crate::{
    frame::{parse as fparse, FrameGenerator, FrameParser},
    types::Id,
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

/// A simple authentication object
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientAuth {
    pub client_id: Id,
    pub token: Id,
}

impl ClientAuth {
    pub fn new(client_id: Id) -> Self {
        Self {
            client_id,
            token: Id::random(),
        }
    }
}

impl Debug for ClientAuth {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ClientAuth {{ id: {}, token: _ }}", self.client_id)
    }
}

impl FrameGenerator for ClientAuth {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.client_id.generate(buf)?;
        self.token.generate(buf)?;
        Ok(())
    }
}

impl FrameGenerator for Option<ClientAuth> {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Some(auth) => auth.generate(buf)?,
            None => buf.push(0),
        }
        Ok(())
    }
}

impl FrameParser for ClientAuth {
    type Output = Option<Self>;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, client_id) = fparse::maybe_id(input)?;
        let (input, token) = fparse::maybe_id(input)?;

        match (client_id, token) {
            (Some(client_id), Some(token)) => Ok((input, Some(Self { client_id, token }))),
            (None, None) => Ok((input, None)),
            _ => unreachable!(
                "Probably reachable, but probably we should return a Result<Option<T>> here"
            ),
            // let res = match (addr, token) {
            //     (Some(addr), Some(token)) => Ok(Self { addr, token }),
            //     (None, Some(_)) => Err(MicroframeError::MissingFields(&["addr"])),
            //     (Some(_), None) => Err(MicroframeError::MissingFields(&["token"])),
            //     (None, None) => Err(MicroframeError::MissingFields(&["addr", "token"])),
            // }
            // .map_err(|e| RatmanError::Microframe(e));
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
