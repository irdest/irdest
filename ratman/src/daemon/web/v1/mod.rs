use crate::Router;
use openapi_type::OpenapiType;
use serde::Serialize;
use tide::http::mime;
use tide::{prelude::*, Request, Response};

#[derive(Debug, Serialize, OpenapiType)]
struct Addr {
    pub id: String,
    pub is_local: bool,
}

#[derive(Debug, Serialize, OpenapiType)]
struct GetAddrsResponse {
    pub addrs: Vec<Addr>,
}

pub async fn get_addrs(req: Request<Router>) -> tide::Result {
    let addrs = req
        .state()
        .known_addresses()
        .await
        .into_iter()
        .map(|(addr, is_local)| Addr {
            id: format!("{:}", addr),
            is_local,
        })
        .collect::<Vec<Addr>>();
    Ok(Response::builder(200)
        .content_type(mime::JSON)
        .body(json!(GetAddrsResponse { addrs }))
        .build())
}

pub async fn get_openapi(_req: Request<Router>) -> tide::Result {
    // I would like there to be a better way to generate this JSON blob.
    //
    // Unfortunately, the structs in the `openapiv3` crate are quite extensive, and don't
    // have builders, so constructing them by hand makes this function many times longer,
    // with nested, verbose `let mut x = X::default(); x.y = Some(y); x.z = "Z".into();`
    // boilerplate, and much harder to understand or work with for everyone involved.
    //
    // Until we can figure out something less painful, let's just construct JSON by hand.
    Ok(Response::builder(200)
        .content_type(mime::JSON)
        .body(json!({
            "paths": {

                "/addrs": {
                    "get": {
                        "tags": ["addr"],
                        "summary": "List known addresses",
                        "operationId": "getAddrs",
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": GetAddrsResponse::schema().schema,
                                    },
                                },
                            },
                        },
                    },
                },

            },
            "components": {
                "schemas": {
                    "Addr": Addr::schema().schema,
                },
            },

            "tags": [{
                "name": "addr",
                "description": "Addresses",
            }],

            "info": {
                "title": "ratmand",
                "version": "1.0",
            },
            "servers": [{ "url": "/api/v1" }],
            "openapi": "3.0.2",
        }))
        .build())
}
