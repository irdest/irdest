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
