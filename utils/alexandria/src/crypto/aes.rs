//! Symmetric cipher utilities
//!
//! These functions are only used to bootstrap the unlocking process
//! for the database user table.  For all other cryptographic
//! operations, see the `asym` module instead.

use crate::{
    crypto::CipherText,
    error::{Error, Result},
    io::wire::{Decode, Encode},
};
use keybob::{Key as KeyBuilder, KeyType};
use serde::{de::DeserializeOwned, Serialize};
use sodiumoxide::crypto::secretbox::{gen_nonce, open, seal, Key as SodiumKey, Nonce};

/// An AES encryption key backed by libsodium
pub(crate) struct Key {
    inner: SodiumKey,
}

impl Key {
    pub fn from_pw(pw: &str, salt: &str) -> Self {
        todo!()
    }

    fn seal<T: Encode<T>, Serialize>(&self, data: &T) -> Result<CipherText> {
        let nonce = gen_nonce();
        let encoded = data.encode()?;
        let data = seal(&encoded, &nonce, &self.inner);

        Ok(CipherText {
            nonce: nonce.0.iter().cloned().collect(),
            data,
        })
    }

    fn open<T: Decode<T>, DeserializeOwned>(&self, data: &CipherText) -> Result<T> {
        let CipherText {
            ref nonce,
            ref data,
        } = data;
        let nonce = Nonce::from_slice(nonce.as_slice()).ok_or(Error::InternalError {
            msg: "Failed to read nonce!".into(),
        })?;
        let clear =
            open(data.as_slice(), &nonce, &self.inner).map_err(|_| Error::InternalError {
                msg: "Failed to decrypt data".into(),
            })?;
        Ok(T::decode(&clear)?)
    }
}

#[test]
fn key_is_key() {
    let k1 = KeyBuilder::from_pw(KeyType::Aes128, "password", "salt");
    let k2 = KeyBuilder::from_pw(KeyType::Aes128, "password", "salt");
    assert_eq!(k1, k2);
}
