// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::Router;
use std::path::Path;
use tide::{http::mime, prelude::*, utils::After, Request, Response};

pub mod v1;

#[derive(rust_embed::RustEmbed)]
#[folder = "dashboard/dist"]
struct DashboardAssets;

async fn serve_dashboard(req: Request<Router>) -> tide::Result {
    let path = {
        let path = req.url().path();
        if path == "/" {
            "index.html"
        } else {
            path.strip_prefix('/').unwrap_or(path)
        }
    };
    let asset = DashboardAssets::get(path)
        .or_else(|| DashboardAssets::get("index.html"))
        .ok_or_else(|| tide::Error::from_str(404, format!("not found: /{:}", path)))?;
    Ok(Response::builder(200)
        .content_type(
            mime::Mime::from_extension(
                Path::new(&path)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
            .unwrap_or(mime::PLAIN),
        )
        .body(&asset.data[..])
        .build())
}

pub async fn start(router: Router, bind: &str, port: u16) -> tide::Result<()> {
    // Create a new application with state
    let mut app = tide::with_state(router);

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
    app.at("/api/v1/addrs").get(v1::get_addrs);

    // Let the dashboard handle any routes we don't recognise.
    app.at("/").get(serve_dashboard);
    app.at("/*").get(serve_dashboard);

    // Then asynchronously run the web server
    let fut = app.listen(format!("{}:{}", bind, port));
    async_std::task::spawn(async move {
        match fut.await {
            Ok(_) => {}
            Err(e) => error!(
                "An error was encountered while running ratmand webui: {:?}",
                e
            ),
        }
    });

    Ok(())
}
