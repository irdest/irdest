use libratman::types::{Address, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub recipient: Recipient,
    pub listeners: BTreeSet<Address>,
}
