use chrono::{DateTime, Utc};
use libratman::{
    frame::carrier::RouteDataV1,
    types::{Address, Id},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteData {
    peer: Address,
    link_id: Id,
    route_id: Id,
    route: RouteEntry,
}

/// Represent a single route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteEntry {
    pub data: RouteDataV1,
    pub state: RouteState,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Describe the state of a given route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouteState {
    Active,
    Idle,
    Lost,
}
