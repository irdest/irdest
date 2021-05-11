//! Irdest webui service library

use irpc_sdk::RpcSocket;
use tide::Server;

pub struct WebServer {
    inner: Server<()>,
    rpc: RpcSocket,
}

impl WebServer {
    pub fn new(rpc: RpcSocket) -> Self {
        let mut inner = Server::new();

        // Add routes here
        inner.at("/").get(|_| Ok("Hello world"));

        Self { inner, rpc }
    }

    pub async fn listen(self, bind: &str) {
        self.inner.listen(bind).await;
    }
}
