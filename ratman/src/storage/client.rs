//! Store clients and their addresses in an SQL database

use crate::{api::BaseClient, storage::addrs::StorageAddress};
use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use libratman::types::Id;
use serde::{Deserialize, Serialize};

/// This type is similar to BaseClient, but capable of being stored on
/// disk
#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct StorageClient {
    pub(crate) id: Id,
    pub(crate) token: Id,
    pub(crate) addrs: Vec<StorageAddress>,
    pub(crate) last_connection: DateTime<Utc>,
}

impl StorageClient {
    pub(crate) fn new(id: Id, client: &Arc<BaseClient>) -> Self {
        Self {
            id,
            token: client.token,
            addrs: client.addrs.clone(),
            last_connection: **client.last_connection.get_ref(),
        }
    }
}
