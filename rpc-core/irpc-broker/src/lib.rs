//! An extensible rpc message broker for the libqaul ecosystem.

#[macro_use]
extern crate tracing;

mod map;
use map::ServiceMap;

mod proto;

use async_std::{net::TcpStream, task};
use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io, RpcSocket, DEFAULT_BROKER_ADDRESS as ADDRESS,
};
use std::sync::Arc;

/// Hold the main broker state
pub struct Broker {
    _sock: Arc<RpcSocket>,
    _conns: Arc<ServiceMap>,
}

impl Broker {
    pub async fn new() -> RpcResult<Arc<Self>> {
        let (addr, port) = default_socket_path();
        Self::bind(addr, port).await
    }

    pub async fn bind(addr: &str, port: u16) -> RpcResult<Arc<Self>> {
        let _conns = ServiceMap::new();

        // Create a new RpcSocket that listens for new connections and
        // runs the `reader_loop` for each of them.  This means that
        // each client on the RPC bus can be expected to have their
        // own reader_loop.
        let _sock = {
            let con = Arc::clone(&_conns);
            RpcSocket::server(addr, port, con, |stream, data| reader_loop(stream, data)).await?
        };

        Ok(Arc::new(Self { _sock, _conns }))
    }
}

/// Continously read from a TcpStream
fn reader_loop(mut stream: TcpStream, map: Arc<ServiceMap>) {
    task::block_on(async {
        // First make sure that we receive a registry message and upgrade our connection
        if let Err(e) = proto::register_service(&map, &mut stream).await {
            error!("Failed to register service: '{}'; dropping connection", e);
            return;
        }

        // Then create a run-loop where we continuously handle incoming messages
        debug!("Listening for incoming messages");
        loop {
            // Some errors are fatal, others are not
            match handle_packet(&mut stream, &map).await {
                Ok(()) => {}
                Err(RpcError::ConnectionFault(msg)) => {
                    error!("Connection suffered a fatal error: {}", msg);
                    break;
                }
                Err(e) => {
                    warn!("Error while accepting packet: {}; dropping stream", e);
                    continue;
                }
            }
        }
    });
}

/// The main logic loop of the broker
async fn handle_packet(s: &mut TcpStream, map: &Arc<ServiceMap>) -> RpcResult<()> {
    let msg = io::recv(s).await?;
    match msg.to.as_str() {
        ADDRESS => proto::handle_sdk_command(s, map, msg).await,
        _ => proto::proxy_message(s, map, msg).await,
    }
}

#[cfg(test)]
pub(crate) fn setup_logging() {
    use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};
    let filter = EnvFilter::try_from_env("QAUL_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::DEBUG.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("async_io=error".parse().unwrap())
        .add_directive("polling=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    info!("Initialised logger: welcome to qaul-hubd!");
}

#[async_std::test]
async fn test_registration() -> RpcResult<()> {
    use irpc_sdk::{Capabilities, Service};
    setup_logging();

    // Create a broker to connect to
    let broker = Broker::new().await?;
    let (addr, port) = default_socket_path();

    // Create a client socket and service and register it
    let mut socket = RpcSocket::connect(addr, port).await?;
    let mut service = Service::new("org.irdest.test", 1, "A simple test service");
    service
        .register(&mut socket, Capabilities::basic_json())
        .await?;
    let id = service.id().unwrap();

    let broker_id = broker
        ._conns
        .hash_id(&"org.irdest.test".to_string())
        .await
        .unwrap();

    assert_eq!(id, broker_id);

    // FIXME: this currently breaks the test
    // service.shutdown().await?;

    Ok(())
}
