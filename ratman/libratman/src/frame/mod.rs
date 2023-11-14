//! Various Frame abstractions for Irdest tools
//!
//! A frame is a self-contained packet with a unified way of parsing
//! an incoming data stream, usually ending with a payload length,
//! which should be loaded after the given header.

pub mod carrier;
pub mod micro;

use crate::Result;
use nom::IResult;

/// A utility trait that represents a parsable frame entity
///
/// This trait is a slim wrapper around the nom parsing
/// infrastructure, aka a top-level parser still needs to map nom
/// errors to RatmanErrors.
pub trait FrameParser {
    type Output;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output>;
}

/// A utility trait that represents a serialisable frame entity
///
/// This trait should be implemented for frame sub-types to avoid code
/// duplication when serialising entities.  Additionally this trait
/// consumes the given frame to avoid duplication.
pub trait FrameGenerator {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()>;
}
