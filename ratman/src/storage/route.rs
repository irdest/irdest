use crate::routes::EpNeighbourPair;
use chrono::{DateTime, Utc};
use libratman::{
    api::types::PeerEntry,
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

    /// Parse the stored connection entries for an address and construct a `PeerEntry`
    ///
    /// This function iterates over the set of available links quite often, and
    /// can probably be optimised to do that less.
    pub fn make_peer_entry(&self) -> PeerEntry {
        let addr = self.peer;
        let first_connection = self.link_data.iter().fold(Utc::now(), |acc, (_, entry)| {
            if entry.first_seen < acc {
                entry.first_seen
            } else {
                acc
            }
        });
        let last_connection = self
            .link_data
            .iter()
            .fold(first_connection, |acc, (_, entry)| {
                if entry.last_seen > acc {
                    entry.last_seen
                } else {
                    acc
                }
            });

        let active = self
            .link_data
            .iter()
            .find(|(_, entry)| entry.state == RouteState::Active)
            .is_some();

        PeerEntry {
            addr,
            first_connection,
            last_connection,
            active,
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
