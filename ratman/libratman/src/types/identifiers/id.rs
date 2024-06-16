// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{types::identifiers::ID_LEN, RatmanError};
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    Deserialize, Serialize, Serializer,
};
use std::{
    fmt::{self, Debug, Display, Formatter},
    string::ToString,
};

#[cfg(feature = "metrics")]
use prometheus_client::encoding::text::Encode;

/// A random identifier for the Irdest ecosystem
///
/// Internally an ID is 32 bytes long.  This data can either be:
///
/// 1. A random identifier for any given resource.
///
/// 2. An ed25519 public key, backed by a corresponding private key.
///
/// API functions that _require_ an `Id` to be backed by a private key MUST wrap
/// it via the [`Address`](super::address::Address) type.
///
/// For every consumer that simply wants to identify a unique object, with a
/// reasonable amount of entropy to avoid collisions, this type is sufficient.
/// You can generate it via [`random()`](Id::random) function, or by hashing a
/// piece of data ([`from_hash()])(Id::from_hash).  The hash function used is
/// blake2.
#[derive(Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Ident32([u8; ID_LEN]);

impl Debug for Ident32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<ID: {}>", hex::encode_upper(self))
    }
}

impl Display for Ident32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            hex::encode_upper(self)
                .as_bytes()
                .chunks(4)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .join("-")
        )
    }
}

#[cfg(feature = "metrics")]
impl Encode for Ident32 {
    fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        write!(w, "{:}", self)
    }
}

impl Ident32 {
    /// Create an identity from the first 16 bytes of a vector
    ///
    /// This function will panic, if the provided vector isn't long
    /// enough, but extra data will simply be discarded.
    pub fn truncate(bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        assert!(bytes.len() >= ID_LEN);

        Self(
            bytes
                .into_iter()
                .enumerate()
                .take(ID_LEN)
                .fold([0; ID_LEN], |mut buf, (i, u)| {
                    buf[i] = *u;
                    buf
                }),
        )
    }

    /// Create an identity from a byte slice
    ///
    /// If the slice is not long enough (or too long), an appropriate
    /// error will be returned, instead of panicking.
    pub fn try_from_bytes(buf: &[u8]) -> crate::Result<Self> {
        if buf.len() != ID_LEN {
            return Err(RatmanError::WrongIdentifierLength(ID_LEN, buf.len()));
        }

        Ok(Self::from_bytes(buf))
    }

    /// Create an identity from an exactly length-matched byte slice
    ///
    /// This function will panic, if the provided slice isn't exactly
    /// the length of the underlying identity implementation (see
    /// `ID_LEN`)
    pub fn from_bytes(buf: &[u8]) -> Self {
        assert_eq!(buf.len(), ID_LEN);
        Self(
            buf.into_iter()
                .enumerate()
                .fold([0; ID_LEN], |mut buf, (i, u)| {
                    buf[i] = *u;
                    buf
                }),
        )
    }

    pub fn pretty_string(&self) -> String {
        let base = self.to_string();

        let sections = base.split(|c| c == '-').into_iter().collect::<Vec<&str>>();

        let head = &sections[0..2];
        let tail = &sections[14..16];

        format!("[{}:{}::  ::{}:{}]", head[0], head[1], tail[0], tail[1])
    }

    pub fn from_string(s: &String) -> Self {
        let v: Vec<u8> = s
            .split("-")
            .map(|s| {
                hex::decode(s).expect(
                    "Don't call from_string() on input that was not serialised by to_string()!",
                )
            })
            .collect::<Vec<Vec<u8>>>()
            .into_iter()
            .flatten()
            .collect();
        Self::from_bytes(&v)
    }

    /// Create an un-initialised identifier which is all-zero
    pub fn uninit() -> Self {
        Self([0; 32])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Copy the contents out as a slice
    #[inline]
    pub fn slice(&self) -> [u8; ID_LEN] {
        self.0
    }

    /// Create an identity using a digest function
    ///
    /// This allows you to pass arbitrary length data which will
    /// result in a precise ID length data output.  The hash function
    /// is the cryptographic [blake2] cipher, so it can be used to
    /// turn secrets into identity information.
    ///
    /// [blake2]: https://blake2.net/
    pub fn with_digest<'vec, V: Into<&'vec Vec<u8>>>(vec: V) -> Self {
        use blake2::{
            digest::{Update, VariableOutput},
            VarBlake2b,
        };

        let mut hasher = VarBlake2b::new(ID_LEN).unwrap();
        hasher.update(vec.into());
        Self::truncate(hasher.finalize_boxed())
    }

    /// Generate a new random Identity
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut buf = [0; ID_LEN];

        // TODO: replace this with an async generator which always has
        // SOME IDs ready to use and only runs occasionally to re-fill
        // a buffer.  That way we can rely on quick ID access without
        // random latency spikes if we end up generating a few IDs
        // that aren't suitble for being used as IDs.
        loop {
            // IDs are not allowed to have ZERO bytes because
            // otherwise a whole bunch of crap breaks elsewhere.
            rng.fill_bytes(&mut buf);

            // If we can't find a ZERO byte, we break from the loop
            if buf.iter().find(|x| *x == &0).is_none() {
                break;
            }

            trace!("ID generation failed, retrying...");
        }

        Self(buf)
    }

    /// Returns an iterator over the bytes of the identity
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a u8> {
        self.0.iter()
    }

    /// A cmp function which always loops over the full byte array
    ///
    /// This is done to avoid timing attacks based on invalid tokens.
    /// This mechanism isn't currently used as the default
    /// PartialEq/Eq implementation because the `Address` type relies
    /// on fast comparisons, which this does not provide.
    // TODO: can we do this with simd ??  Feels like yes.
    pub fn compare_constant_time(&self, other: &Self) -> bool {
        let mut valid = true;
        let mut ctr = 0;
        while ctr < ID_LEN {
            valid &= self.0[ctr] == other.0[ctr];
            ctr += 1;
        }
        valid
    }
}

/// Implement RAW `From` binary array
impl From<[u8; ID_LEN]> for Ident32 {
    fn from(i: [u8; ID_LEN]) -> Self {
        Self(i)
    }
}

/// Implement RAW `From` binary (reference) array
impl From<&[u8; ID_LEN]> for Ident32 {
    fn from(i: &[u8; ID_LEN]) -> Self {
        Self(i.clone())
    }
}

/// Implement binary array `From` RAW
impl From<Ident32> for [u8; ID_LEN] {
    fn from(i: Ident32) -> Self {
        i.0
    }
}

/// Implement binary array `From` RAW reference
impl From<&Ident32> for [u8; ID_LEN] {
    fn from(i: &Ident32) -> Self {
        i.0.clone()
    }
}

/// Implement RAW identity to binary array reference
impl AsRef<[u8]> for Ident32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Iterator for iterating over `Identity`
pub struct Iter {
    index: usize,
    ident: Ident32,
}

impl Iterator for Iter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.ident.0.get(self.index).map(|byte| *byte);
        self.index += 1;
        ret
    }
}

impl IntoIterator for Ident32 {
    type Item = u8;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            index: 0,
            ident: self,
        }
    }
}

impl Serialize for Ident32 {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if ser.is_human_readable() {
            ser.serialize_str(&self.to_string())
        } else {
            ser.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Ident32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        struct IdentityVisitor;

        impl IdentityVisitor {
            fn from_str<E: Error>(v: &str) -> Result<Ident32, E> {
                let v: Vec<u8> = v
                    .split("-")
                    .map(|s| hex::decode(s).map_err(|e| E::custom(e)))
                    // I don't like this way of propagating errors up but the alternative
                    // is a for loop which i also don't like
                    .collect::<Result<Vec<Vec<u8>>, E>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                Self::from_bytes(&v)
            }

            fn from_bytes<E: Error, V: AsRef<[u8]>>(v: V) -> Result<Ident32, E> {
                let v = v.as_ref();
                if v.len() != ID_LEN {
                    return Err(E::custom(format!(
                        "Expected {} bytes, got {}",
                        ID_LEN,
                        v.len()
                    )));
                }

                Ok(Ident32(v.iter().enumerate().take(ID_LEN).fold(
                    [0; ID_LEN],
                    |mut buf, (i, u)| {
                        buf[i] = *u;
                        buf
                    },
                )))
            }
        }

        impl<'de> Visitor<'de> for IdentityVisitor {
            type Value = Ident32;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                write!(
                    f,
                    "Either a {l} byte array or a hex string representing {l} bytes",
                    l = ID_LEN
                )
            }

            fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Self::from_str(v)
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Self::from_str(&v)
            }

            fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
                Self::from_bytes(v)
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Self::from_bytes(v)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element::<u8>()? {
                    v.push(b);
                }

                Self::from_bytes(v)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(IdentityVisitor)
        } else {
            deserializer.deserialize_bytes(IdentityVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bincode;
    use serde_json;

    #[test]
    #[cfg(not(features = "aligned"))]
    fn json_serde() {
        let s = b"Yes, we will make total destroy.";
        let i = Ident32::truncate(&s.to_vec());
        let v = serde_json::to_string(&i).unwrap();
        assert_eq!(
            v,
            "\"5965-732C-2077-6520-7769-6C6C-206D-616B-6520-746F-7461-6C20-6465-7374-726F-792E\""
        );
        let i2 = serde_json::from_str(&v).unwrap();
        assert_eq!(i, i2);
    }

    #[test]
    #[cfg(not(features = "aligned"))]
    fn bincode_serde() {
        let s = b"Yes, we will make total destroy.";
        let i = Ident32::truncate(&s.to_vec());
        let v: Vec<u8> = bincode::serialize(&i).unwrap();
        assert_eq!(
            v,
            vec![
                32, 0, 0, 0, 0, 0, 0, 0, 89, 101, 115, 44, 32, 119, 101, 32, 119, 105, 108, 108,
                32, 109, 97, 107, 101, 32, 116, 111, 116, 97, 108, 32, 100, 101, 115, 116, 114,
                111, 121, 46
            ],
        );
        let i2 = bincode::deserialize(&v).unwrap();
        assert_eq!(i, i2);
    }

    #[test]
    #[cfg(features = "aligned")]
    fn sized() {
        assert_eq!(super::ID_LEN, size_of::<usize>());
    }

    /// This is the default length
    #[test]
    #[cfg(not(features = "aligned"))]
    fn sized() {
        assert_eq!(super::ID_LEN, 32);
    }
}
