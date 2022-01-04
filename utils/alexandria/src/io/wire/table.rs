//! Protobuf wrapper module

use protobuf::Message;

use crate::io::{error::Result, proto::table as proto};
use std::io::{Read, Write};

/// Un-encrypted table header containing column name and type data
pub struct TableHeader {
    inner: proto::TableHeader,
}

impl TableHeader {
    /// Create a new table header with a set of column names and types
    ///
    /// This function will only be called when creating a new table
    /// and initialised with a row-count of `0`.
    pub fn new(columns: Vec<String>, column_types: Vec<Vec<u8>>) -> Self {
        let mut inner = proto::TableHeader::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_rows(0);
        inner.set_cols(columns.into());
        inner.set_col_types(column_types.into());
        Self { inner }
    }

    /// Increment the row counter
    pub fn add_row(&mut self) {
        let r = self.inner.get_rows() + 1;
        self.inner.set_rows(r);
    }
}

/// Wrapper for an unencrypted row header
pub struct RowHeader {
    inner: proto::RowHeader,
}

impl RowHeader {
    /// Create a new RowHeader from index and length data
    ///
    /// The length MUST be derived from the encrypted RowData stream
    /// before writing to a chunk.
    pub fn new(idx: u64, len: u64) -> Self {
        let mut inner = proto::RowHeader::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_index(idx);
        inner.set_length(len);
        Self { inner }
    }
    /// Create a new RowHeader wrapper from a reader
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let inner = proto::RowHeader::parse_from_reader(reader)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        self.inner.write_length_delimited_to_writer(writer)?;
        Ok(())
    }

}

/// Wrapper for an unencrypted row data section
pub struct RowData {
    inner: proto::RowData,
}

impl RowData {
    /// Create new row data from a set of encoded column data
    pub fn new(cols: Vec<Vec<u8>>) -> Self {
        let mut inner = proto::RowData::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_cols(cols.into());
        Self { inner }
    }

    /// Create a new RowHeader wrapper from a reader
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let inner = proto::RowData::parse_from_reader(reader)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        self.inner.write_length_delimited_to_writer(writer)?;
        Ok(())
    }

}
