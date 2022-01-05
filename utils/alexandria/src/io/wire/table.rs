//! Protobuf wrapper module

use crate::{
    crypto::{CipherText, CryEngineHandle, CryReqPayload, CryRespPayload, ResponsePayload},
    io::{
        error::Result,
        proto::table as proto,
        wire::traits::{FromEncrypted, FromReader, ToEncrypted, ToWriter},
    },
};
use protobuf::Message;

/// Un-encrypted table header containing column name and type data
#[derive(Debug, PartialEq)]
pub struct TableHeader {
    inner: proto::TableHeader,
}

impl TableHeader {
    /// Create a new table header with a set of column names and types
    ///
    /// This function will only be called when creating a new table
    /// and initialised with a row-count of `0`.
    pub(crate) fn new(columns: Vec<String>, column_types: Vec<Vec<u8>>) -> Self {
        let mut inner = proto::TableHeader::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_rows(0);
        inner.set_cols(columns.into());
        inner.set_col_types(column_types.into());
        Self { inner }
    }

    /// Increment the row counter
    pub(crate) fn add_row(&mut self) {
        let r = self.inner.get_rows() + 1;
        self.inner.set_rows(r);
    }
}

impl ToWriter for TableHeader {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}
impl FromReader for TableHeader {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::TableHeader::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}

impl ToEncrypted for TableHeader {}
impl FromEncrypted for TableHeader {}

/// Wrapper for an unencrypted row header
#[derive(Debug, PartialEq)]
pub(crate) struct RowHeader {
    inner: proto::RowHeader,
}

impl RowHeader {
    /// Create a new RowHeader from index and length data
    ///
    /// The length MUST be derived from the encrypted RowData stream
    /// before writing to a chunk.
    pub(crate) fn new(idx: u64, len: u64) -> Self {
        let mut inner = proto::RowHeader::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_index(idx);
        inner.set_length(len);
        Self { inner }
    }
}

impl ToWriter for RowHeader {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}
impl FromReader for RowHeader {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::RowHeader::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}

impl ToEncrypted for RowHeader {}
impl FromEncrypted for RowHeader {}

/// Wrapper for an unencrypted row data section
#[derive(Debug, PartialEq)]
pub(crate) struct RowData {
    inner: proto::RowData,
}

impl RowData {
    /// Create new row data from a set of encoded column data
    pub(crate) fn new(cols: Vec<Vec<u8>>) -> Self {
        let mut inner = proto::RowData::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_cols(cols.into());
        Self { inner }
    }
}

impl ToWriter for RowData {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}
impl FromReader for RowData {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::RowData::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}

impl ToEncrypted for RowData {}
impl FromEncrypted for RowData {}
