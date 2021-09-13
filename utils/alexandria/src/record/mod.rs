//! Multi-mode record module
//!
//! Alexandria is a "multi-mode" database, meaning that records can
//! have vastly different shapes.  The two main distinguishing
//! categories are:
//!
//! * Document type
//! * Table type
//!
//! ## Document records
//!
//! A document record is fundamentally a key-value store.  This type
//! of record may support versioning, meaning that deletions are not
//! applied to the main body but rather mark keys as "superceded".
//! This supports rolling back data to previous states, but also
//! linearly increases the size of records over time.
//!
//! This type of record is recomended for data collections that can
//! easily be mapped into a HashMap in your program.  Furthermore,
//! document records must (at the moment) be loaded into memory in
//! full, meaning that memory limitations are a concearn.
//!
//! ## Table records
//!
//! A table record is modelled after traditional SQL databases.  They
//! have a schema, a set of indices (as well as per-table search
//! caches), and rows of data.  Inserted data will be appended to the
//! table, while deletions will _actually_ delete data from the table.
//! Tables can also be partially loaded into memory (i.e. their schema
//! and search caches), with queries loading parts of the table as
//! needed.
//!
//! This type of record is recommended for complex datastructures, or
//! large collections that you may want to iterate through gradually.
//!
//! ## Structure of this module
//!
//! Because Alexandria must support both these modes, this module is
//! split into two sub-modules implementing the different record
//! strategies.  The `Record` top-level type is a wrapper around the
//! mandatory record header and can then distinguish between the
//! underlying types.

use crate::utils::{Id, TagSet};
use chrono::{DateTime, Utc};

pub struct Record {
    header: Header,
    dheader: DataHeader,
    data: Body,
}

pub struct Header {
    id: Id,
    tags: TagSet,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

/// A data-specific header
///
/// Stores information about on-disk formats to allow chunk-loading
pub(crate) struct DataHeader {
    size: usize,
    chunks: usize,
}

/// A body-type determinant
///
/// Gain access to body-specific APIs by matching on this type.
///
/// ```rust,ignore
/// match body {
///    Doc(api) => api.update(),
///    _ => unreachable!()
/// }
/// ```

pub enum Body {
    Doc,
    Table,
}
