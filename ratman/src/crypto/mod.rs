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
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use std::convert::TryInto;
use x25519_dalek::{PublicKey as X25519Pubkey, SharedSecret, StaticSecret as X25519Secret};

/// An ed25519 keypair
///
/// The public key represents an address on the Irdest network
pub struct Keypair {
    inner: ed25519_dalek::Keypair,
}

impl Keypair {
    fn to_expanded(&self) -> ExpandedSecretKey {
        ExpandedSecretKey::from(&self.inner.secret)
    }
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
        let secret = SecretKey::generate(&mut OsRng {});
        let public = PublicKey::from(&secret);
        let id = Identity::from_bytes(public.as_bytes());

        let mut map = self.inner.write().await;
        map.insert(
            id,
            Keypair {
                inner: ed25519_dalek::Keypair { public, secret },
            },
        );
        id
    }

    pub async fn diffie_hellman(&self, _self: Identity, peer: Identity) -> Option<SharedSecret> {
        let map = self.inner.read().await;
        let self_keypair = map.get(&_self)?;

        // Here we're taking an private key on the edwards curve and
        // transform it to a private key on the montgomery curve.
        // This is done via the `ExpandedSecretKey` type, which does
        // this transformation internally, while also adding another
        // 32 bytes of "nonce", which we discard.
        let self_x25519_secret = {
            let self_expanded = self_keypair.to_expanded();
            let self_montgomery: [u8; 32] = self_expanded.to_bytes()[..32].try_into().ok()?;
            X25519Secret::from(self_montgomery)
        };

        // The public key represents a compressed point on the edwards
        // curve, which can be decompressed, and then transformed to a
        // point on the montgomery curve.
        let peer_x25519_public = {
            let peer_compressed = CompressedEdwardsY::from_slice(peer.as_bytes());
            let peer_edwards = peer_compressed.decompress().unwrap();
            let peer_montgomery = peer_edwards.to_montgomery();
            X25519Pubkey::from(peer_montgomery.to_bytes())
        };

        Some(self_x25519_secret.diffie_hellman(&peer_x25519_public))
    }

    pub async fn sign_manifest(&self, _self: Identity, msg: &[u8]) -> Option<Signature> {
        let map = self.inner.read().await;
        let self_keypair = map.get(&_self)?;
        Some(self_keypair.inner.sign(msg))
    }

    pub fn verify_manifest(&self, peer: Identity, msg: &[u8], signature: Signature) -> Option<()> {
        let peer_pubkey = PublicKey::from_bytes(peer.as_bytes()).ok()?;
        peer_pubkey.verify(msg, &signature).ok()
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
    println!("A->B {:?}", alice_to_bob.as_bytes());

    // Computer B
    let bob_to_alice = store.diffie_hellman(bob, alice).await.unwrap();
    println!("A->B {:?}", bob_to_alice.as_bytes());

    // Outside the universe
    assert_eq!(alice_to_bob.as_bytes(), bob_to_alice.as_bytes());
}

#[async_std::test]
async fn manifest_signature() {
    let store = Keystore::new();

    // Computer A
    let alice = store.create_address().await;
    let manifest = vec![7, 6, 9, 6, 5, 8, 7, 4, 3, 6, 8, 8, 5, 5, 7, 8, 5, 5, 87];
    let signature = store
        .sign_manifest(alice, manifest.as_slice())
        .await
        .unwrap();

    // Computer B
    assert_eq!(
        store.verify_manifest(alice, manifest.as_slice(), signature),
        Some(())
    )
}
