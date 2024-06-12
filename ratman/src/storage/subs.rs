use async_eris::ReadCapability;
use libratman::types::{Address, LetterheadV1, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub recipient: Recipient,
    pub listeners: BTreeSet<Address>,
    pub missed_items: BTreeMap<Recipient, Vec<(LetterheadV1, ReadCapability)>>,
}
