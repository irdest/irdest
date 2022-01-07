//! Record data module

use crate::utils::{Id, TagSet};
use chrono::{DateTime, Utc};
use id::Identity;

// pub struct Record {
//     header: Header,
//     dheader: DataHeader,
//     data: Body,
// }

pub struct Header {
    id: Id,
    tags: TagSet,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

/// A cached version of a table
pub struct Table {}

/// Some piece of data found in the leaf position of a record
pub enum LeafType {
    /// A boolean `true` or `false` value
    Bool(bool),
    /// A 64-bit signed integer number
    Integer(i64),
    /// A double-precision floating point number
    Double(f64),
    /// A UTF-8 string
    String(String),
    /// Any kind of arbitrary binary data
    Binary(Vec<u8>),
    /// A reference to another table entry
    InternalRef(u64),
    /// Reference to an entry in a different table
    ExternalRef(Identity, u64),
}

/// A single row of data
pub struct Row {
    /// The index of this row
    idx: u64,
    /// Column data
    cols: Vec<LeafType>,
}

/// Provide a simple iterator over a set of rows
pub struct RowIterator {}

impl RowIterator {
    pub fn new() -> Self {
        todo!()
    }

    pub fn next(&self) -> () {
        ()
    }

    pub fn jump_to(&self, idx: u64) {}
}
