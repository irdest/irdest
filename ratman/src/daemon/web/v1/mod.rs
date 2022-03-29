use crate::Router;
use serde::Serialize;
use tide::http::mime;
use tide::{prelude::*, Request, Response};

#[derive(Debug, Serialize)]
struct Addr {
    pub id: usize,
    pub addr: String,
    pub is_local: bool,
}

pub async fn get_addrs(req: Request<Router>) -> tide::Result {
    let addrs = req
        .state()
        .known_addresses()
        .await
        .into_iter()
        .enumerate()
        .map(|(id, (addr, is_local))| Addr {
            id: id + 1,
            addr: format!("{:}", addr),
            is_local,
        })
        .collect::<Vec<Addr>>();
    Ok(Response::builder(200)
        .content_type(mime::JSON)
        .body(json!({ "addrs": addrs }))
        .build())
}
