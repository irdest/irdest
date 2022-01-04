use id::Identity;

use crate::{
    crypto::pkcry::{PubKey, SecKey},
    Locked,
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) struct KeyStore {
    /// The root keypair for a database
    root: (PubKey, SecKey),
    /// Per-user public key store
    pubs: Locked<BTreeMap<Identity, Arc<PubKey>>>,
    /// Pur-user secret key store
    subs: Locked<BTreeMap<Identity, Arc<SecKey>>>,
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

    pub async fn get_pair(self: &Arc<Self>, user: &Identity) -> Option<(Arc<PubKey>, Arc<SecKey>)> {
        let pubkey = Arc::clone(self.pubs.read().await.get(user).as_ref()?);
        let seckey = Arc::clone(self.subs.read().await.get(user).as_ref()?);
        Some((pubkey, seckey))
    }
}
