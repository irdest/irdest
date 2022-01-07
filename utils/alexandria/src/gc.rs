//! Database-internal record garbage collection
//!
//! This module integrates with [io](crate::io) and
//! [cache](crate::cache) to provide a single unified front for
//! garbage collection and record data fetching.

use crate::io::Record;

///
pub(crate) struct Gc {}

/// A single record entry in the garbage collection system
pub(crate) struct GcEntry {
    r: Record,
}
