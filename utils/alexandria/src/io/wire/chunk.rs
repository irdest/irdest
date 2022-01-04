//! Protobuf wrapper module

use crate::crypto::{CipherText, CryEngineHandle, CryReqPayload, CryRespPayload, ResponsePayload};
use crate::io::{
    error::Result,
    proto::chunk as proto,
    wire::{
        self,
        encrypted::{Encrypted, EncryptedChunk},
    },
};
use id::Identity;
use protobuf::Message;
use std::io::{Read, Write};

/// Un-encrypted chunk header at the start of a chunk file
#[derive(Debug, PartialEq)]
pub struct ChunkHeader {
    inner: proto::Header,
}

impl ChunkHeader {
    pub(crate) fn new(max_len: u64) -> Self {
        let mut inner = proto::Header::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_maxLen(max_len);
        Self { inner }
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

    /// Take an encrypted chunk and turn it into a clear chunk header
    pub(crate) async fn from_encrypted(
        e: EncryptedChunk,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let header = e.header();
        let nonce = header.nonce().to_vec();
        let data = header.data().to_vec();

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

    /// Create a new RowHeader wrapper from a reader
    pub(crate) fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let buf = wire::read_with_length(reader)?;
        let inner = proto::Header::parse_from_bytes(&buf)?;
        Ok(Self { inner })
    }

    /// Write length-prepended encoding to writer stream
    pub(crate) fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        let buf = self.inner.write_to_bytes()?;
        wire::write_with_length(writer, &buf)?;
        Ok(())
    }
}
