//! Manage the libqaul, service and ratman states

use crate::cfg::Config;
use irdest_core::{helpers::Directories, Irdest};
use netmod_tcp::{Endpoint, Mode};
use ratman::Router;
use std::{fs::File, io::Read, sync::Arc};

#[allow(unused)]
pub(crate) struct State {
    pub qaul: Arc<Irdest>,
    pub router: Arc<Router>,
}

impl State {
    /// Create a new run state
    pub(crate) async fn new(cfg: &Config) -> State {
        let ep = Endpoint::new(
            &cfg.addr,
            cfg.port,
            "qaul-hubd",
            match cfg.mode.as_str() {
                "dynamic" => Mode::Dynamic,
                _ => Mode::Static,
            },
        )
        .await
        .unwrap();

        let mut buf = String::new();
        let mut peersfd = File::open(&cfg.peers).unwrap();
        peersfd.read_to_string(&mut buf).unwrap();

        let peers = buf.split("\n").map(|s| s.to_string()).collect();
        ep.add_peers(peers).await.unwrap();

        let router = Router::new();
        router.add_endpoint(ep).await;

        let qaul = Irdest::new(
            Arc::clone(&router),
            Directories::new("irdest-hubd").unwrap(),
        );

        Self { qaul, router }
    }
}
