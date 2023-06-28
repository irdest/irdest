use async_std::net::TcpListener;
use libratman::types::Result;
use std::net::SocketAddr;

use self::connection::ConnectionManager;

mod client;
mod connection;
mod io;

/// Client API manager
pub struct ClientApiManager {
    connections: ConnectionManager,
}

impl ClientApiManager {
    ///
    pub async fn start(bind: SocketAddr) -> Result<Self> {
        info!("Listening for API connections on socket {:?}", bind);
        let listener = TcpListener::bind(bind).await?;

        todo!()
    }
}
