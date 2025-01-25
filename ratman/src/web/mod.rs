// SPDX-FileCopyrightText: 2022, 2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod v1;
use libratman::axum::response::Html;
use libratman::axum_embed::ServeEmbed;
use libratman::tokio::spawn;
use rust_embed::RustEmbed;
pub use v1::NeighbourEntry;

use crate::context::RatmanContext;
use libratman::axum::{routing::get, serve, Router};
use libratman::tokio::net::TcpListener;
use libratman::{RatmanError, Result as RatmanResult};
use prometheus_client::registry::Registry;
use std::sync::Arc;

struct WebState {
    router: Arc<RatmanContext>,
}

#[derive(RustEmbed, Clone)]
#[folder = "dashboard/dist"]
struct DashboardAssets;

pub async fn start(
    router: Arc<RatmanContext>,
    _registry: Registry,
    bind_addr: String,
) -> RatmanResult<()> {
    // Metrics and logging for HTTP requests.
    // let instrument = middleware::Instrument::default();
    // instrument.register_metrics(&mut registry);

    let state = WebState { router };

    let serve_dashboard = ServeEmbed::<DashboardAssets>::new();

    // Build a router and attach some routes to it
    let router = Router::new()
        .nest_service("/", serve_dashboard)
        //.route("/", get(hello_world))
        .route("/api/v1/addrs", get(v1::get_addrs))
        .route("/api/v1/peers", get(v1::get_peers))
        .with_state(Arc::new(state));

    let listener = TcpListener::bind(bind_addr.clone())
        .await
        .map_err(|e| RatmanError::Io(e))?;

    info!("The ratmand web dashboard is available at http://{bind_addr}");
    serve(listener, router).await.unwrap();

    Ok(())
}
