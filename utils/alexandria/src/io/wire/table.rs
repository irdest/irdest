//! Protobuf wrapper module

use protobuf::Message;
use tracing::callsite::Identifier;

use crate::{
    crypto::{CipherText, CryEngineHandle, CryReqPayload, CryRespPayload, ResponsePayload},
    io::{
        error::Result,
        proto::table as proto,
        wire::{self, encrypted::Encrypted, read_with_length},
    },
};
use id::Identity;
use std::io::{Read, Write};

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

    /// Take an `Encrypted` and turn it into a `TableHeader`
    pub(crate) async fn from_encrypted(
        e: Encrypted<'_>,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let nonce = e.nonce().to_vec();
        let data = e.data().to_vec();

        let (request, rx) = CryReqPayload::decrypt(user, CipherText { nonce, data });
        cry.tx.send(request).await;
        let clear_vec = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Clear(vec) => vec,
                _ => unreachable!(),
            },
            _ => panic!("FIXME: Failed to decrypt"),
        };

        Self::from_reader(&mut clear_vec.as_slice())
    }

    /// Turn this type into an `Encrypted`
    pub(crate) async fn to_encrypted(
        &self,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Encrypted<'_>> {
        let mut payload = vec![];
        self.to_writer(&mut payload)?;

        let (request, rx) = CryReqPayload::encrypt(user, payload);
        cry.tx.send(request).await;
        let CipherText { nonce, data } = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Encrypted(ciphered) => ciphered,
                _ => unreachable!(),
            },
            _ => panic!("FIXME: failed to encrypt!"),
        };

        Ok(Encrypted::new(nonce, data))
    }

    /// Increment the row counter
    pub(crate) fn add_row(&mut self) {
        let r = self.inner.get_rows() + 1;
        self.inner.set_rows(r);
    }
    /// Create a new RowHeader wrapper from a reader
    pub(crate) fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let buf = wire::read_with_length(reader)?;
        let inner = proto::TableHeader::parse_from_bytes(&buf)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub(crate) fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        let buf = self.inner.write_to_bytes()?;
        wire::write_with_length(writer, &buf)?;
        Ok(())
    }
}

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

    /// Take an `Encrypted` and turn it into a `RowHeader`
    pub(crate) async fn from_encrypted(
        e: Encrypted<'_>,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let nonce = e.nonce().to_vec();
        let data = e.data().to_vec();

        let (request, rx) = CryReqPayload::decrypt(user, CipherText { nonce, data });
        cry.tx.send(request).await;
        let clear_vec = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Clear(vec) => vec,
                _ => unreachable!(),
            },
            _ => panic!("FIXME: Failed to decrypt"),
        };

        Self::from_reader(&mut clear_vec.as_slice())
    }

    /// Turn this type into an `Encrypted`
    pub(crate) async fn to_encrypted(
        &self,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Encrypted<'_>> {
        let mut payload = vec![];
        self.to_writer(&mut payload)?;

        let (request, rx) = CryReqPayload::encrypt(user, payload);
        cry.tx.send(request).await;
        let CipherText { nonce, data } = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Encrypted(ciphered) => ciphered,
                _ => unreachable!(),
            },
            _ => panic!("FIXME: failed to encrypt!"),
        };

        Ok(Encrypted::new(nonce, data))
    }

    /// Create a new RowHeader wrapper from a reader
    pub(crate) fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let buf = wire::read_with_length(reader)?;
        let inner = proto::RowHeader::parse_from_bytes(&buf)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub(crate) fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        let buf = self.inner.write_to_bytes()?;
        wire::write_with_length(writer, &buf)?;
        Ok(())
    }
}

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

    /// Take an `Encrypted` and turn it into a `RowData`
    pub(crate) async fn from_encrypted(
        e: Encrypted<'_>,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let nonce = e.nonce().to_vec();
        let data = e.data().to_vec();

        let (request, rx) = CryReqPayload::decrypt(user, CipherText { nonce, data });
        cry.tx.send(request).await;
        let clear_vec = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Clear(vec) => vec,
                _ => unreachable!(),
            },
            _ => panic!("FIXME: Failed to decrypt"),
        };

        Self::from_reader(&mut clear_vec.as_slice())
    }

    /// Turn this type into an `Encrypted`
    pub(crate) async fn to_encrypted(
        &self,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Encrypted<'_>> {
        let mut payload = vec![];
        self.to_writer(&mut payload)?;

        let (request, rx) = CryReqPayload::encrypt(user, payload);
        cry.tx.send(request).await;
        let CipherText { nonce, data } = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Encrypted(ciphered) => ciphered,
                _ => unreachable!(),
            },
            e => panic!("FIXME: failed to encrypt: {:?}", e),
        };

        Ok(Encrypted::new(nonce, data))
    }

    /// Create a new RowHeader wrapper from a reader
    pub(crate) fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let buf = wire::read_with_length(reader)?;
        let inner = proto::RowData::parse_from_bytes(&buf)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub(crate) fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        let buf = self.inner.write_to_bytes()?;
        wire::write_with_length(writer, &buf)?;
        Ok(())
    }
}
