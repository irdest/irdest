use crate::routes::EpNeighbourPair;
use chrono::{DateTime, Utc};
use libratman::{
    frame::carrier::RouteDataV1,
    types::{Address, Ident32},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteData {
    pub peer: Address,
    pub link_id: Vec<EpNeighbourPair>,
    pub link_data: BTreeMap<EpNeighbourPair, RouteEntry>,
    pub route_id: Ident32,
}

impl RouteData {
    pub fn local(addr: Address) -> Self {
        Self {
            peer: addr,
            link_id: Vec::new(),
            link_data: BTreeMap::new(),
            route_id: Ident32::random(),
        }
    }
}

/// Represent a single route
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RouteEntry {
    pub data: RouteDataV1,
    pub state: RouteState,
    pub ping: Duration,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Describe the state of a given route
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RouteState {
    Active,
    Idle,
    Lost,
}
