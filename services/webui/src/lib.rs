//! Irdest webui service library

use irpc_sdk::{Capabilities, Identity, RpcSocket, Service};
use std::sync::Arc;
use tide::Server;
use irdest_sdk::IrdestSdk;
use tide::Request;

pub struct WebServer {
    app: Server<State>,
    rpc: Arc<RpcSocket>,
    service_id: Identity,
}

#[derive(Clone)]
struct State {
    sdk: IrdestSdk,
}

impl WebServer {
    pub async fn new(addr: &str, port: u16) -> Self {
        let mut serv = Service::new("org.irdest.webui", 1, "Webui.");
        let rpc = RpcSocket::connect(addr, port).await.unwrap();
        
        let mut app = tide::with_state(State {
            sdk: IrdestSdk::connect(&serv).unwrap()
        });

        let service_id = serv
            .register(&rpc, Capabilities::basic_json())
            .await
            .unwrap();

        // Add routes here
        app.at("/").get(|req: Request<State>| async {
            // irdest_sdk.users().create("blörp").await;
            Ok("Hello world")
        });

        Self {
            app,
            rpc,
            service_id,
        }
    }

    pub async fn listen(self, bind: &str) {
        self.app.listen(bind).await.unwrap();
    }
}
