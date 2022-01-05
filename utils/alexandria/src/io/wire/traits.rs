use crate::{
    crypto::{CipherText, CryEngineHandle, CryReqPayload, CryRespPayload, ResponsePayload},
    io::wire::{read_with_length, write_with_length, Encrypted, Result},
};
use id::Identity;
use std::io::{Read, Write};
use tracing::callsite::Identifier;

/// Take any wire type and turn it into an `Encrypted` type with some
/// async cryptographic magic.
#[async_trait::async_trait]
pub(crate) trait ToEncrypted: ToWriter {
    async fn to_encrypted(&self, user: Identity, cry: CryEngineHandle) -> Result<Encrypted> {
        let mut payload = vec![];
        self.to_writer(&mut payload)?;

        let (req, rx) = CryReqPayload::encrypt(user, payload);
        cry.tx.send(req).await;
        let CipherText { nonce, data } = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Encrypted(ciphered) => ciphered,
                _ => unreachable!(),
            },
            e => {
                // idk throw a more scalable error or something
                panic!("Failed to encrypt: {:?}", e);
            }
        };

        Ok(Encrypted::new(nonce, data))
    }
}

/// Take any wire type and encode it with length prepended
pub(crate) trait ToWriter {
    /// Small utility function that must be implemented for each type
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Blanket implementation for length delimited writing
    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        let buf = self.to_bytes()?;
        write_with_length(writer, &buf)?;
        Ok(())
    }
}

/// Take an encrypted wire type and turn it into any other type with
/// some async cryptographic magic
#[async_trait::async_trait]
pub(crate) trait FromEncrypted: FromReader + Sized {
    async fn from_encrypted(
        e: Encrypted<'_>,
        user: Identity,
        cry: CryEngineHandle,
    ) -> Result<Self> {
        let nonce = e.nonce().to_vec();
        let data = e.data().to_vec();

        let (req, rx) = CryReqPayload::decrypt(user, CipherText { nonce, data });
        cry.tx.send(req).await;
        let clear_vec = match rx.recv().await {
            Ok(CryRespPayload { status, payload }) if status == 0 => match payload {
                ResponsePayload::Clear(vec) => vec,
                _ => unreachable!(),
            },
            e => {
                // idk same as above
                panic!("Failed to decrypt: {:?}", e);
            }
        };

        Self::from_reader(&mut clear_vec.as_slice())
    }
}

pub(crate) trait FromReader: Sized {
    /// Small utility function that must be implemented for each type
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self>;

    /// Blanket implementation for length delimited reading
    fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let buf = read_with_length(reader)?;
        FromReader::new_from_bytes(&buf)
    }
}
