//! Ratman cryptography module
//!
//! This module uses the x25519-dalek crate to computer a shared
//! diffie-helman secret between two addresses on the network.  This
//! shared key can then be used to encode and encrypt ERIS blocks.
//!
//! An address corresponds to the public key of a key pair, where the
//! private key is not shared outside the router.

// Utility imports
use crate::storage::addrs::StorageAddress;
use libratman::{tokio::sync::RwLock, types::Address};
use rand::{rngs::OsRng, thread_rng, RngCore};
use std::{collections::BTreeMap, convert::TryInto};

// Cryptography imports
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey, Signature, Signer, Verifier};
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

/// Encrypt a data chunk with a shared secret and random nonce
///
/// Returns the encrypted chunk as well as the selected nonce, which
/// must be provided for decoding
pub fn encrypt_chunk<const L: usize>(shared_key: &SharedSecret, chunk: &mut [u8; L]) -> [u8; 12] {
    let mut nonce = [0; 12];
    thread_rng().fill_bytes(&mut nonce);

    let mut cipher = ChaCha20::new(&(*shared_key.as_bytes()).into(), &nonce.into());
    cipher.apply_keystream(chunk.as_mut_slice());
    nonce
}

/// Decrypt a data chunk with a shared secret and nonce
pub fn decrypt_chunk(
    shared_key: &SharedSecret,
    nonce: [u8; 12],
    mut encrypted_chunk: Vec<u8>,
) -> Vec<u8> {
    let mut cipher = ChaCha20::new(&(*shared_key.as_bytes()).into(), &nonce.into());
    cipher.apply_keystream(&mut encrypted_chunk);
    encrypted_chunk
}

pub struct Keystore {
    inner: RwLock<BTreeMap<Address, Keypair>>,
    pw: String,
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
            pw: "using this as a user password for now :)".into(),
        }
    }

    pub async fn add_address(&self, id: Address, key_data: &[u8]) -> Option<()> {
        let mut map = self.inner.write().await;
        let secret = SecretKey::from_bytes(key_data).ok()?;
        let public = PublicKey::from(&secret);

        map.insert(
            id,
            Keypair {
                inner: ed25519_dalek::Keypair { public, secret },
            },
        );
        Some(())
    }

    /// Create a new address keypair
    pub async fn create_address(&self) -> Address {
        let secret = SecretKey::generate(&mut OsRng {});
        let public = PublicKey::from(&secret);
        let id = Address::from_bytes(public.as_bytes());

        let mut map = self.inner.write().await;
        map.insert(
            id,
            Keypair {
                inner: ed25519_dalek::Keypair { public, secret },
            },
        );
        id
    }

    /// Get all currently registered local addresses and encrypted keys
    pub async fn get_all(&self) -> Vec<StorageAddress> {
        self.inner
            .read()
            .await
            .iter()
            .map(|(id, kp)| StorageAddress::new(*id, &kp))
            .collect()
    }

    pub async fn diffie_hellman(&self, _self: Address, peer: Address) -> Option<SharedSecret> {
        let map = self.inner.read().await;
        let self_keypair = map.get(&_self)?;

        // Here we're taking a private key on the edwards curve and
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

        // Finally we can perform a diffie hellman exchange between
        // the private sender and public recipient address keys
        Some(self_x25519_secret.diffie_hellman(&peer_x25519_public))
    }

    /// Sign a payload with your secret key
    pub async fn sign_message(&self, _this: Address, msg: &[u8]) -> Option<Signature> {
        let map = self.inner.read().await;
        let self_keypair = map.get(&_this)?;
        Some(self_keypair.inner.sign(msg))
    }

    /// Verify the signature of a payload with a peer's public key (address)
    pub fn verify_message(&self, peer: Address, msg: &[u8], signature: Signature) -> Option<()> {
        let peer_pubkey = PublicKey::from_bytes(peer.as_bytes()).ok()?;
        peer_pubkey.verify(msg, &signature).ok()
    }
}

#[libratman::tokio::test]
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

#[libratman::tokio::test]
async fn manifest_signature() {
    let store = Keystore::new();

    // Computer A
    let alice = store.create_address().await;
    let manifest = vec![7, 6, 9, 6, 5, 8, 7, 4, 3, 6, 8, 8, 5, 5, 7, 8, 5, 5, 87];
    let signature = store
        .sign_message(alice, manifest.as_slice())
        .await
        .unwrap();

    // Computer B
    assert_eq!(
        store.verify_message(alice, manifest.as_slice(), signature),
        Some(())
    )
}

#[libratman::tokio::test]
async fn diffie_hellman_chacha20() {
    let store = Keystore::new();
    let alice = store.create_address().await;
    let bob = store.create_address().await;

    let mut message = vec![7, 6, 9, 6, 5, 8, 7, 4, 3, 6, 8, 8, 5, 5, 7, 8, 5, 5, 87];
    let check = message.clone();
    let message_len = message.len();
    let alice_to_bob = store.diffie_hellman(alice, bob).await.unwrap();

    let nonce = [0; 12];
    let mut cipher1 = ChaCha20::new(&(*alice_to_bob.as_bytes()).into(), &nonce.into());
    cipher1.apply_keystream(message.as_mut_slice());

    eprintln!("Encrypted message reads: {:?}", message);

    let bob_from_alice = store.diffie_hellman(bob, alice).await.unwrap();
    let mut cipher2 = ChaCha20::new(&(*bob_from_alice.as_bytes()).into(), &nonce.into());
    cipher2.apply_keystream(message.as_mut_slice());
    assert_eq!(message, check);
}
