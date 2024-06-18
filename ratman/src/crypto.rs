//! Ratman cryptography module
//!
//! This module uses the x25519-dalek crate to computer a shared
//! diffie-helman secret between two addresses on the network.  This
//! shared key can then be used to encode and encrypt ERIS blocks.
//!
//! An address corresponds to the public key of a key pair, where the
//! private key is not shared outside the router.

// Utility imports
use crate::storage::{
    addr_key::{AddressData, EncryptedKey},
    MetadataDb,
};
use libratman::{
    tokio::sync::Mutex,
    types::{AddrAuth, Address, Ident32},
    ClientError, RatmanError, Result,
};
use once_cell::sync::Lazy;
use rand::{rngs::OsRng, thread_rng, RngCore};
use std::{collections::BTreeMap, convert::TryInto, ffi::CString, sync::Arc};

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
    pub(crate) inner: ed25519_dalek::Keypair,
}

impl Keypair {
    pub fn new(secret: SecretKey) -> Self {
        let public = PublicKey::from(&secret);
        Self {
            inner: ed25519_dalek::Keypair { secret, public },
        }
    }

    fn to_expanded(&self) -> ExpandedSecretKey {
        ExpandedSecretKey::from(&self.inner.secret)
    }
}

/// Encrypt a data chunk with a shared secret and random nonce
///
/// The provided chunk is encrypted in place and the selected nonce (which must
/// be provided to decrypt) is returned
pub fn encrypt_chunk<const L: usize>(shared_key: &SharedSecret, chunk: &mut [u8; L]) -> [u8; 12] {
    encrypt_raw(shared_key.as_bytes(), chunk)
}

pub fn encrypt_raw(secret: &[u8; 32], data: &mut [u8]) -> [u8; 12] {
    let mut nonce = [0; 12];
    thread_rng().fill_bytes(&mut nonce);

    let mut cipher = ChaCha20::new(&(*secret).into(), &nonce.into());
    cipher.apply_keystream(data);
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

pub fn decrypt_raw(secret: &[u8; 32], nonce: [u8; 12], encrypted_data: &mut Vec<u8>) {
    let mut cipher = ChaCha20::new(&(*secret).into(), &nonce.into());
    cipher.apply_keystream(encrypted_data.as_mut_slice());
}

pub fn diffie_hellman(self_keypair: &Keypair, peer: Address) -> Option<SharedSecret> {
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

// Cache decrypted address keys and shared secret keys for particular streams
static KEY_CACHE: Lazy<Mutex<BTreeMap<Ident32, Keypair>>> =
    Lazy::new(|| Mutex::new(BTreeMap::default()));
static SHARED_CACHE: Lazy<Mutex<BTreeMap<(Address, Address), SharedSecret>>> =
    Lazy::new(|| Mutex::new(BTreeMap::default()));

pub fn list_addr_keys(meta_db: &Arc<MetadataDb>) -> Vec<Address> {
    meta_db
        .addrs
        .iter()
        .into_iter()
        .map(|(addr, _)| Address::from_string(&addr))
        .collect()
}

pub fn get_addr_key(meta_db: &Arc<MetadataDb>, addr: Address, auth: AddrAuth) -> Result<Keypair> {
    let key_data = match meta_db
        .addrs
        .get(&addr.to_string())?
        .ok_or(RatmanError::Encoding(libratman::EncodingError::Encryption(
            "Address key was deleted!".into(),
        )))? {
        AddressData::Local(e, _name) => e,
        AddressData::Space(e, _name) => e,
        AddressData::Remote => unreachable!("called open_addr_key with a remote key"),
    };

    let mut decrypted_key = key_data.encrypted.clone();
    decrypt_raw(
        auth.token.as_bytes().try_into().unwrap(),
        key_data.nonce,
        &mut decrypted_key,
    );

    let secret_key = SecretKey::from_bytes(&decrypted_key)
        .map_err::<RatmanError, _>(|_| ClientError::InvalidAuth.into())?;
    let public_key = PublicKey::from(&secret_key);

    let computed_addr = Address::from_bytes(public_key.as_bytes());
    assert_eq!(computed_addr, addr);

    Ok(Keypair::new(secret_key))
}

pub fn create_addr_key(
    meta_db: &Arc<MetadataDb>,
    name: Option<CString>,
    space: Option<Ident32>,
) -> Result<(Address, AddrAuth)> {
    let (secret, addr) = if let Some(secret_data) = space {
        let secret = SecretKey::from_bytes(secret_data.as_bytes()).unwrap();
        let public = PublicKey::from(&secret);
        let addr = Address::from_bytes(public.as_bytes());

        (secret, addr)
    } else {
        let secret = SecretKey::generate(&mut OsRng {});
        let public = PublicKey::from(&secret);
        let addr = Address::from_bytes(public.as_bytes());

        (secret, addr)
    };

    // Generate a public-private keypair
    let client_auth = AddrAuth::new();

    let mut encrypted_secret = *secret.as_bytes();
    let nonce = encrypt_raw(
        client_auth.token.as_bytes().try_into().unwrap(),
        &mut encrypted_secret,
    );

    let address_data = if space.is_some() {
        AddressData::Space(
            EncryptedKey {
                encrypted: encrypted_secret.to_vec(),
                nonce,
            },
            name,
        )
    } else {
        AddressData::Local(
            EncryptedKey {
                encrypted: encrypted_secret.to_vec(),
                nonce,
            },
            name,
        )
    };

    meta_db.addrs.insert(addr.to_string(), &address_data)?;

    Ok((addr, client_auth))
}

/// Destroy the local address key data
pub fn destroy_addr_key(meta_db: &Arc<MetadataDb>, addr: Address) -> Result<()> {
    meta_db.addrs.remove(addr.to_string())?;
    Ok(())
}

/// Encrypt a chunk for an ongoing session.
///
/// Panics: This function will panic if start_stream is not called first!
pub async fn encrypt_chunk_for_key<const L: usize>(
    _meta_db: &Arc<MetadataDb>,
    self_addr: Address,
    target_addr: Address,
    _auth: AddrAuth,
    chunk: &mut [u8; L],
) -> [u8; 12] {
    let map = SHARED_CACHE.lock().await;
    let shared = map
        .get(&(self_addr, target_addr))
        .expect("Shared secret not found in cache; must call `start_stream` first!");
    encrypt_chunk(shared, chunk)
}

/// Sign a payload with a cached secret key
pub async fn sign_message(auth: AddrAuth, msg: &[u8]) -> Result<Signature> {
    Ok(KEY_CACHE
        .lock()
        .await
        .get(&auth.token)
        .ok_or(RatmanError::Encoding(libratman::EncodingError::Encryption(
            "no key cached for this AddrAuth token".to_owned(),
        )))?
        .inner
        .sign(msg))
}

/// Verify the signature of a payload with a peer's public key (address)
pub fn verify_message(peer: Address, msg: &[u8], signature: Signature) -> Option<()> {
    let peer_pubkey = PublicKey::from_bytes(peer.as_bytes()).ok()?;
    peer_pubkey.verify(msg, &signature).ok()
}

// #[libratman::tokio::test]
// async fn shared_key() {
//     let store = Keystore::new();

//     // Computer A
//     let alice = store.create_address().await;

//     // Computer B
//     let bob = store.create_address().await;

//     // Computer A
//     let alice_to_bob = store.diffie_hellman(alice, bob).await.unwrap();
//     println!("A->B {:?}", alice_to_bob.as_bytes());

//     // Computer B
//     let bob_to_alice = store.diffie_hellman(bob, alice).await.unwrap();
//     println!("A->B {:?}", bob_to_alice.as_bytes());

//     // Outside the universe
//     assert_eq!(alice_to_bob.as_bytes(), bob_to_alice.as_bytes());
// }

// #[libratman::tokio::test]
// async fn manifest_signature() {
//     let store = Keystore::new();

//     // Computer A
//     let alice = store.create_address().await;
//     let manifest = vec![7, 6, 9, 6, 5, 8, 7, 4, 3, 6, 8, 8, 5, 5, 7, 8, 5, 5, 87];
//     let signature = store
//         .sign_message(alice, manifest.as_slice())
//         .await
//         .unwrap();

//     // Computer B
//     assert_eq!(
//         store.verify_message(alice, manifest.as_slice(), signature),
//         Some(())
//     )
// }

// #[libratman::tokio::test]
// async fn diffie_hellman_chacha20() {
//     let store = Keystore::new();
//     let alice = store.create_address().await;
//     let bob = store.create_address().await;

//     let mut message = vec![7, 6, 9, 6, 5, 8, 7, 4, 3, 6, 8, 8, 5, 5, 7, 8, 5, 5, 87];
//     let check = message.clone();
//     let message_len = message.len();
//     let alice_to_bob = store.diffie_hellman(alice, bob).await.unwrap();

//     let nonce = [0; 12];
//     let mut cipher1 = ChaCha20::new(&(*alice_to_bob.as_bytes()).into(), &nonce.into());
//     cipher1.apply_keystream(message.as_mut_slice());

//     eprintln!("Encrypted message reads: {:?}", message);

//     let bob_from_alice = store.diffie_hellman(bob, alice).await.unwrap();
//     let mut cipher2 = ChaCha20::new(&(*bob_from_alice.as_bytes()).into(), &nonce.into());
//     cipher2.apply_keystream(message.as_mut_slice());
//     assert_eq!(message, check);
// }
