use crate::{
    crypto::CipherText,
    error::{Error, Result},
    io::{Decode, Encode},
};
use serde::{de::DeserializeOwned, Serialize};
use sodiumoxide::crypto::box_::{self, Nonce, PublicKey, SecretKey};

/// Generate a new random pk pair
pub(crate) fn keypair() -> (PubKey, SecKey) {
    let (p, s) = box_::gen_keypair();
    (PubKey { inner: p }, SecKey { inner: s })
}

/// A wrapper around an NaCl public key
pub struct PubKey {
    inner: PublicKey,
}

impl PubKey {
    pub(crate) fn seal(&self, data: &[u8], auth: &SecKey) -> CipherText {
        let non = box_::gen_nonce();
        let data = box_::seal(&data, &non, &self.inner, &auth.inner);
        let nonce = non.0.iter().cloned().collect();
        CipherText { nonce, data }
    }
}

/// A wrapper around an NaCl secret key
pub struct SecKey {
    inner: SecretKey,
}

impl SecKey {
    pub(crate) fn open(&self, data: &CipherText, auth: &PubKey) -> Result<Vec<u8>> {
        let CipherText {
            ref nonce,
            ref data,
        } = data;

        let nonce =
            Nonce::from_slice(&nonce.as_slice()).ok_or(Error::internal("Failed to read nonce!"))?;

        let clear = box_::open(data.as_slice(), &nonce, &auth.inner, &self.inner)
            .map_err(|_| Error::internal("Failed to decrypt data"))?;

        Ok(clear)
    }
}

#[test]
fn seal_and_open_string() {
    let (p, s) = keypair();
    let data1: String = "Encrypting repo. A little, secure horse cry. at the perfect bowl".into();

    let ct = p.seal(&data1.encode().unwrap(), &s);
    let buf: Vec<u8> = s.open(&ct, &p).unwrap();
    let data2 = String::decode(&buf).unwrap();

    assert_eq!(data1, data2);
}
