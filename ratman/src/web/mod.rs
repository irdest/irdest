// SPDX-FileCopyrightText: 2022, 2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::context::RatmanContext;
use prometheus_client::registry::Registry;
use std::{path::Path, sync::Arc};
use tide::{http::mime, prelude::*, utils::After, Request, Response};

pub mod middleware;
pub mod v1;

pub type State = Arc<StateData>;

pub struct StateData {
    pub router: Arc<RatmanContext>,
    pub registry: Registry,
}

#[derive(rust_embed::RustEmbed)]
#[folder = "dashboard/dist"]
struct DashboardAssets;

async fn serve_dashboard(req: Request<State>) -> tide::Result {
    let path = {
        let path = req.url().path();
        if path == "/" {
            "index.html"
        } else {
            path.strip_prefix('/').unwrap_or(path)
        }
    };
    let (asset, mtype) = DashboardAssets::get(path)
        .map(|ass| {
            let mtype = mime::Mime::from_extension(
                Path::new(&path)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
            .unwrap_or(mime::PLAIN);
            (ass, mtype)
        })
        .or_else(|| DashboardAssets::get("index.html").map(|ass| (ass, mime::HTML)))
        .ok_or_else(|| tide::Error::from_str(404, format!("not found: /{:}", path)))?;
    Ok(Response::builder(200)
        .content_type(mtype)
        .body(&asset.data[..])
        .build())
}

async fn serve_metrics(req: Request<State>) -> tide::Result {
    let mut body = Vec::new();
    prometheus_client::encoding::text::encode(&mut body, &req.state().registry).unwrap();
    let response = tide::Response::builder(200)
        .body(body)
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .build();
    Ok(response)
}

pub async fn start(
    router: Arc<RatmanContext>,
    mut registry: Registry,
    bind_addr: String,
) -> tide::Result<()> {
    // Metrics and logging for HTTP requests.
    let instrument = middleware::Instrument::default();
    instrument.register_metrics(&mut registry);

    // Create a new application with state
    let mut app = tide::with_state(Arc::new(StateData { router, registry }));
    app.with(instrument);

    // Convert errors into a form Ember.js can understand.
    app.with(After(|mut res: Response| async {
        if let Some(err) = res.error() {
            let status = err.status();
            let body = json!({ "errors": [{"detail": format!("{:}", err)}] });

            // The above are immutable borrows, here are the mutable ones.
            res.set_status(status);
            res.set_body(body);
        }
        Ok(res)
    }));

    // Attach some routes to it.
    app.at("/api/v1/openapi.json").get(v1::get_openapi);
    app.at("/api/v1/addrs").get(v1::get_addrs);

    app.at("/_/metrics").get(serve_metrics);

    // Let the dashboard handle any routes we don't recognise.
    app.at("/").get(serve_dashboard);
    app.at("/*").get(serve_dashboard);

    // Then asynchronously run the web server
    let fut = app.listen(bind_addr);
    async_std::task::spawn(async move {
        match fut.await {
            Ok(_) => {}
            Err(e) => error!(
                "An error was encountered while running ratmand dashboard: {:?}",
                e
            ),
        }
    });

    Ok(())
}
