//! Message dispatch module
//!
//! Accepts a stream of ERIS blocks on one end and returns a sequence
//! of correctly sliced CarrierFrame's for a particular MTU.
//!
//! Accepts a sequence of CarrierFrame's and re-assembles them back
//! into full ERIS blocks.

mod collector;
pub(crate) use collector::BlockCollector;

mod slicer;
pub(crate) use slicer::{BlockSlicer, StreamSlicer};
