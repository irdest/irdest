use crate::{
    crypto::{
        pkcry::{PubKey, SecKey},
        Hid,
    },
    Locked,
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) struct KeyStore {
    /// The root keypair for a database
    root: (PubKey, SecKey),
    /// Per-user public key store
    pubs: Locked<BTreeMap<Hid, PubKey>>,
    /// Pur-user secret key store
    subs: Locked<BTreeMap<Hid, SecKey>>,
}

impl KeyStore {
    /// Create a new key tree with the root keypair
    pub fn new(p: PubKey, s: SecKey) -> Arc<Self> {
        Arc::new(Self {
            root: (p, s),
            pubs: Default::default(),
            subs: Default::default(),
        })
    }

    // /// Get access to a secret key
    // pub(crate) async fn get_seckey(self: &Arc<Self>, id: &Hid) -> Option<&SecKey> {
    //     self.subs.read().await.get(id)
    // }

    // /// Get access to a public key
    // pub(crate) async fn get_pubkey(self: &Arc<Self>, id: &Hid) -> Option<&PubKey> {
    //     self.pubs.read().await.get(id)
    // }
}
