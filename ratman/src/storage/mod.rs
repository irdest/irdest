//! This module handles several storage engines.  The backing database is fjall, a
//!
//! - Block storage: keep track of full blocks
//!
//! - Frame storage: keep track of in-flight frames that don't fully assemble a
//! block (yet)
//!
//! - Peer metadata: persistent routing tables
//!
//! -

use crate::{
    crypto::{self, decrypt_raw, encrypt_chunk, encrypt_raw, Keypair},
    journal::page::CachePage,
    storage::{
        addr_key::{AddressData, EncryptedKey},
        link::LinkData,
        route::RouteData,
    },
};
use ed25519_dalek::{PublicKey, SecretKey, Signature};
use fjall::{Keyspace, PartitionCreateOptions};
use libratman::{
    types::{Address, ClientAuth, Id},
    Result,
};
use rand::rngs::OsRng;
use std::{
    borrow::BorrowMut, cell::RefCell, collections::BTreeMap, convert::TryInto, marker::PhantomData,
};
use x25519_dalek::SharedSecret;

pub mod addr_key;
pub mod block;
pub mod link;
pub mod route;

/// Metadata database handle
///
/// This database keeps track of various bits of metadata which aren't directly
/// related to the flow of message streams through an Irdest network.
///
/// - Registered addresses and their encrypted private key information
///
/// - Routing table: keep track of known peers via their links and various
/// metrics like MTU, uptime, and average ping.
///
/// - Link metadata: certain message streams have associations between them, or
/// can be tagged with additional information for importance to prevent them
/// from being cleaned from the journal in case of storage quota limits.
// todo: currently this type handles some encryption operations directly even
// though there's no real reason for this.  Maybe move this code to the crypto
// module?
pub struct MetadataDb {
    db: Keyspace,
    pub addrs: CachePage<AddressData>,
    pub routes: CachePage<RouteData>,
    pub links: CachePage<LinkData>,
}

/// Cache decrypted address keys and shared secret keys for particular streams
thread_local! {
    static KEY_CACHE: RefCell<BTreeMap<Id, Keypair>> = RefCell::new(BTreeMap::default());
    static SHARED_CACHE: RefCell<BTreeMap<(Address, Address), SharedSecret>> = RefCell::new(BTreeMap::default());
}

impl MetadataDb {
    pub fn new(db: Keyspace) -> Result<Self> {
        let addrs = CachePage(
            db.open_partition("meta_addrs", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let routes = CachePage(
            db.open_partition("meta_routes", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let links = CachePage(
            db.open_partition("meta_links", PartitionCreateOptions::default())?,
            PhantomData,
        );

        Ok(Self {
            db,
            addrs,
            routes,
            links,
        })
    }

    pub fn insert_addr_key(&self, client_id: Id) -> Result<(Address, ClientAuth)> {
        // Generate a public-private keypair
        let secret = SecretKey::generate(&mut OsRng {});
        let public = PublicKey::from(&secret);
        let addr = Address::from_bytes(public.as_bytes());

        let client_auth = ClientAuth::new(client_id);

        let mut encrypted_secret = *secret.as_bytes();
        let nonce = encrypt_raw(
            client_auth.token.as_bytes().try_into().unwrap(),
            &mut encrypted_secret,
        );

        self.addrs.insert(
            addr.to_string(),
            &AddressData::Local(EncryptedKey {
                encrypted: encrypted_secret.to_vec(),
                nonce,
            }),
        )?;

        Ok((addr, client_auth))
    }

    /// Decrypt an address key and cache it for the local runner thread
    pub fn open_addr_key(&self, addr: Address, auth: ClientAuth) -> Result<()> {
        let key_data = match self.addrs.get(&addr.to_string())? {
            AddressData::Local(e) => e,
            AddressData::Remote => unreachable!("called open_addr_key with a remote key"),
        };

        let mut decrypted_key = key_data.encrypted.clone();
        decrypt_raw(
            auth.token.as_bytes().try_into().unwrap(),
            key_data.nonce,
            &mut decrypted_key,
        );

        KEY_CACHE.with(|map| {
            map.borrow_mut().insert(
                auth.client_id,
                Keypair::new(SecretKey::from_bytes(decrypted_key.as_slice()).unwrap()),
            );
        });

        Ok(())
    }

    pub fn close_addr_key(&self, auth: ClientAuth) {
        KEY_CACHE.with(|map| {
            map.borrow_mut().remove(&auth.client_id);
        });
    }

    pub fn start_stream(
        &self,
        self_addr: Address,
        target_addr: Address,
        auth: ClientAuth,
    ) -> Result<()> {
        KEY_CACHE.with(|map| {
            let map = map.borrow();
            let decrypted_key = map.get(&auth.client_id).expect("decrypted key not cached!");
            let shared_secret = crypto::diffie_hellman(&decrypted_key, target_addr).unwrap();

            SHARED_CACHE.with(|map| {
                map.borrow_mut()
                    .insert((self_addr, target_addr), shared_secret);
            });
        });

        Ok(())
    }

    pub fn end_stream(&self, self_addr: Address, target_addr: Address) {
        SHARED_CACHE.with(|map| {
            map.borrow_mut().remove(&(self_addr, target_addr));
        })
    }

    pub fn encrypt_chunk_for_key<const L: usize>(
        &self,
        self_addr: Address,
        target_addr: Address,
        auth: ClientAuth,
        chunk: &mut [u8; L],
    ) -> [u8; 12] {
        SHARED_CACHE.with(|map| {
            let map = map.borrow();
            let shared = map.get(&(self_addr, target_addr)).unwrap();
            encrypt_chunk(shared, chunk)
        })
    }

    /// Sign a payload with your secret key
    pub fn sign_message(&self, auth: ClientAuth, msg: &[u8]) -> Option<Signature> {
        KEY_CACHE.with(|map| {
            let map = map.borrow();
            crypto::sign_message(map.get(&auth.token)?, msg)
        })
    }

    /// Verify the signature of a payload with a peer's public key (address)
    pub fn verify_message(&self, peer: Address, msg: &[u8], signature: Signature) -> Option<()> {
        let peer_pubkey = PublicKey::from_bytes(peer.as_bytes()).ok()?;
        crypto::verify_message(Address::from_bytes(peer_pubkey.as_bytes()), msg, signature)
    }
}
