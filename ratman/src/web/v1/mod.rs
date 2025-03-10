use crate::crypto;
use crate::routes::{EpNeighbourPair, NeighbourEntry};
use crate::web::WebState;
use libratman::api::types::PeerEntry;
use libratman::axum::{extract::State, Json};
use libratman::types::{Address, Ident32, Namespace};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

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

pub async fn get_neighbours(
    State(state): State<Arc<WebState>>,
) -> Json<BTreeMap<EpNeighbourPair, NeighbourEntry>> {
    Json(state.router.routes.neighbours().await)
}

pub async fn get_spaces(State(_state): State<Arc<WebState>>) -> Json<BTreeMap<Namespace, Vec<u8>>> {
    Json(BTreeMap::new())
}

pub async fn create_space(State(state): State<Arc<WebState>>) -> Json<(Address, Ident32)> {
    let (pubkey, privkey) = libratman::generate_space_key();

    crypto::create_namespace(&state.router.meta_db, None, pubkey, privkey)
        .await
        .unwrap();
    state
        .router
        .routes
        .register_local_route(pubkey)
        .await
        .unwrap();

    Json((pubkey, privkey))
}

#[derive(Serialize, Deserialize)]
pub struct DiskQuota {
    meta_quota: Option<u64>,
    meta_usage: u64,
    journal_quota: Option<u64>,
    journal_usage: u64,
}

pub async fn get_quotas(State(state): State<Arc<WebState>>) -> Json<DiskQuota> {
    let meta_quota = state.router.meta_db.disk_quota();
    let meta_usage = state.router.meta_db.used_space();

    let journal_quota = state.router.journal.disk_quota();
    let journal_usage = state.router.journal.used_space();

    Json(DiskQuota {
        meta_quota,
        meta_usage,
        journal_quota,
        journal_usage,
    })
}
