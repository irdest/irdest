use crate::{routes::EpNeighbourPair, web::WebState};
use libratman::api::types::PeerEntry;
use libratman::types::Address;
use libratman::{
    axum::{extract::State, Json},
    endpoint::NeighbourMetrics,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

pub async fn get_addrs(State(state): State<Arc<WebState>>) -> Json<Vec<Address>> {
    Json(
        state
            .router
            .meta_db
            .addrs
            .iter()
            .into_iter()
            .map(|(addr, _)| Address::from_string(&addr))
            .collect(),
    )
}

pub async fn get_peers(State(state): State<Arc<WebState>>) -> Json<BTreeMap<Address, PeerEntry>> {
    Json(
        state
            .router
            .routes
            .list_remote()
            .await
            .map(|vec| {
                vec.into_iter()
                    .map(|entry| (entry.addr, entry))
                    .collect::<BTreeMap<Address, PeerEntry>>()
            })
            .unwrap(),
    )
}

#[derive(Serialize, Deserialize)]
pub struct NeighbourEntry {
    pub neighbour_id: String,
    pub neighbour_ping: Duration,
    pub bandwidth: NeighbourMetrics,
    pub buffer: u64,
}

pub async fn get_neighbours(
    State(state): State<Arc<WebState>>,
) -> Json<BTreeMap<EpNeighbourPair, NeighbourEntry>> {
    Json(state.router.routes.neighbours().await)
}
