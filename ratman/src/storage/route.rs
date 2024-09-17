use crate::routes::EpNeighbourPair;
use chrono::{DateTime, Utc};
use libratman::{
    frame::carrier::RouteDataV1,
    types::{Address, Ident32},
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteData {
    pub peer: Address,
    pub link_id: VecDeque<EpNeighbourPair>,
    pub route_id: Ident32,
    pub ping: Duration,
    pub route: Option<RouteEntry>,
}

impl RouteData {
    pub fn local(addr: Address) -> Self {
        Self {
            peer: addr,
            link_id: VecDeque::new(),
            route_id: Ident32::random(),
            ping: Duration::from_millis(rand::random::<u64>() % 10),
            route: None,
        }
    }
}

/// Represent a single route
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RouteEntry {
    pub data: RouteDataV1,
    pub state: RouteState,
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
