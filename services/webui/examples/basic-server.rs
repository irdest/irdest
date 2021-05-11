#![allow(unused)]

use async_std::future::timeout;
use irdest_core::{rpc::RpcServer, Irdest, IrdestRef};
use irdest_webui::WebServer;
use irpc_broker::Broker;
use irpc_sdk::{error::RpcResult, Capabilities, RpcSocket, Service};
use ratman_harness::{millis, sec10, sec5, Initialize, ThreePoint};
use std::{sync::Arc, time::Duration};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

fn parse_log_level() {
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

pub struct TestServer {
    inner: Arc<RpcServer>,
    port: u16,
}

impl TestServer {
    /// Create an RPC server with a random binding
    pub async fn new(ir: IrdestRef, port: u16) -> Self {
        Self {
            port,
            inner: RpcServer::new(ir, "127.0.0.1", port).await.unwrap(),
        }
    }
}

#[allow(unused)]
pub struct RpcState {
    pub tp: ThreePoint<Arc<Irdest>>,
    // Node A state
    rpc_a: TestServer,
    broker_a: Arc<Broker>,
    // Node B state
    // rpc_b: TestServer,
    // broker_b: Arc<Broker>,
}

impl RpcState {
    pub async fn new(a: u16, b: u16) -> Self {
        // parse_log_level(); // If something doesn't work, enable this line!
        let tp = init().await;

        let broker_a = Broker::bind("127.0.0.1", a).await.unwrap();
        let rpc_a = TestServer::new(Arc::clone(&tp.a.1.as_ref().unwrap()), a).await;

        // let broker_b = Broker::bind("127.0.0.1", b).await.unwrap();
        // let rpc_b = TestServer::new(Arc::clone(&tp.b.1.as_ref().unwrap()), b).await;

        Self {
            tp,
            rpc_a,
            broker_a,
            // rpc_b,
            // broker_b,
        }
    }
}

pub async fn zzz(dur: Duration) {
    async_std::task::sleep(dur).await
}

pub async fn make_service(port: u16) -> RpcResult<Service> {
    let socket = RpcSocket::connect("127.0.0.1", port).await?;
    let mut service = Service::new("test", 1, "A test service");
    service
        .register(&socket, Capabilities::basic_json())
        .await?;
    Ok(service)
}

async fn init() -> ThreePoint<Arc<Irdest>> {
    let mut tp = ThreePoint::new().await;
    tp.init_with(|_, arc| Irdest::new(arc));
    tp
}

#[async_std::main]
async fn main() {
    let _state = RpcState::new(6060, 7070).await;

    // Create service
    let s = WebServer::new("127.0.0.1", 6060).await;

    s.listen("127.0.0.1:6061").await;
}
