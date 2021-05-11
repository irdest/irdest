//! Irdest webui service library

use irpc_sdk::{Capabilities, Identity, RpcSocket, Service};
use std::sync::Arc;
use tide::Server;

pub struct WebServer {
    inner: Server<()>,
    rpc: Arc<RpcSocket>,
    service_id: Identity,
}

impl WebServer {
    pub async fn new(addr: &str, port: u16) -> Self {
        let mut inner = Server::new();

        let mut serv = Service::new("org.irdest.webui", 1, "Webui.");
        let rpc = RpcSocket::connect(addr, port).await.unwrap();
        let service_id = serv
            .register(&rpc, Capabilities::basic_json())
            .await
            .unwrap();

        // Add routes here
        inner.at("/").get(|_| async { Ok("Hello world") });

        Self {
            inner,
            rpc,
            service_id,
        }
    }

    pub async fn listen(self, bind: &str) {
        self.inner.listen(bind).await.unwrap();
    }
}
