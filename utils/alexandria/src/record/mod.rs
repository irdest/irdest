//! Record data module

use crate::utils::{Id, TagSet};
use chrono::{DateTime, Utc};

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
pub struct Table {
    
}

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
}

/// A single row of data
pub struct Row {
    /// The index of this row
    idx: u64,
    cols: Vec<LeafType>,
}

/// Provide a simple iterator over a set of rows
pub struct RowIterator {
}

///
impl RowIterator {
    pub fn new() -> Self {
        todo!()
    }

    pub fn next(&self) -> () {
        ()
    }

    pub fn to_index(&self, idx: u64) {
        
    }
}
