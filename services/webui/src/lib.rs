//! Irdest webui service library

mod api;

use irdest_sdk::IrdestSdk;
use irpc_sdk::{Capabilities, Identity, RpcSocket, Service};
use std::sync::Arc;
use tide::Request;
use tide::Server;

pub struct WebServer {
    app: Server<State>,
    rpc: Arc<RpcSocket>,
}

#[derive(Clone)]
struct State {
    service_id: Identity,
    sdk: IrdestSdk,
}

impl WebServer {
    pub async fn new(addr: &str, port: u16) -> Self {
        let mut serv = Service::new("org.irdest.webui", 1, "Webui.");
        let rpc = RpcSocket::connect(addr, port).await.unwrap();

        let service_id = serv
            .register(&rpc, Capabilities::basic_json())
            .await
            .unwrap();

        let mut app = tide::with_state(State {
            service_id,
            sdk: IrdestSdk::connect(&serv).unwrap(),
        });

        // Add routes here
        app.at("/").get(api::auth::create_user);

        Self { app, rpc }
    }

    pub async fn listen(self, bind: &str) {
        self.app.listen(bind).await.unwrap();
    }
}
