//! Irdest webui service library

mod api;

use tide::utils::Before;
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

        app.with(api::auth::LoadUserMiddleware {});
        
        app.at("/auth/register").post(api::auth::register);
        app.at("/auth/login").post(api::auth::login);
        app.at("/auth/verify_token").post(api::auth::verify_token);

        Self { app, rpc }
    }

    pub async fn listen(self, bind: &str) {
        println!("Listening on {}", bind);
        self.app.listen(bind).await.unwrap();
    }
}
