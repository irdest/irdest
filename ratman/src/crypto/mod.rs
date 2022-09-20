//! Ratman cryptography module
//!
//! This module uses the x25519-dalek crate to computer a shared
//! diffie-helman secret between two addresses on the network.  This
//! shared key can then be used to encode and encrypt ERIS blocks.
//!
//! An address corresponds to the public key of a key pair, where the
//! private key is not shared outside the router.

use crate::Identity;
use async_std::sync::RwLock;
use rand_core::OsRng;
use std::{collections::BTreeMap, convert::TryInto};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

pub struct Keypair {
    _pub: PublicKey,
    secr: StaticSecret,
}

pub struct Keystore {
    inner: RwLock<BTreeMap<Identity, Keypair>>,
}

impl Keystore {
    // TODO: load existing keys from database
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    /// Create a new address keypair
    pub async fn create_address(&self) -> Identity {
        let secr = StaticSecret::new(OsRng);
        let _pub = PublicKey::from(&secr);
        let id = Identity::from_bytes(_pub.as_bytes());

        let mut map = self.inner.write().await;
        map.insert(id, Keypair { _pub, secr });
        id
    }

    pub async fn diffie_hellman(&self, _self: Identity, peer: Identity) -> Option<SharedSecret> {
        let map = self.inner.read().await;
        let self_keypair = map.get(&_self)?;

        let peer: [u8; 32] = peer.as_bytes().try_into().ok()?;
        let peer_pubkey = PublicKey::from(peer);
        Some(self_keypair.secr.diffie_hellman(&peer_pubkey))
    }
}

#[async_std::test]
async fn shared_key() {
    let store = Keystore::new();

    // Computer A
    let alice = store.create_address().await;

    // Computer B
    let bob = store.create_address().await;

    // Computer A
    let alice_to_bob = store.diffie_hellman(alice, bob).await.unwrap();

    // Computer B
    let bob_to_alice = store.diffie_hellman(bob, alice).await.unwrap();

    // Outside the universe
    assert_eq!(alice_to_bob.as_bytes(), bob_to_alice.as_bytes());
}
