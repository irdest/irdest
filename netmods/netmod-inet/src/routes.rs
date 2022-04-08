use crate::peer::Peer;
use async_std::sync::{Arc, RwLock};
use std::collections::BTreeMap;

pub(crate) type Target = u64;

pub(crate) struct Routes {
    inner: Arc<RwLock<BTreeMap<Target, Peer>>>,
}
