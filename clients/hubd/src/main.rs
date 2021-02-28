//! # qaul-hubd server
//!
//! A modular and configurable internet overlay server for qaul.

// #[macro_use]
// extern crate tracing;

mod cfg;
mod log;
mod state;

use async_std::{future, task::Poll};
use libqaul::rpc::RpcServer;
use qrpc_broker::Broker;
use state::State;

pub(crate) fn elog<S: Into<String>>(msg: S, code: u16) -> ! {
    tracing::error!("{}", msg.into());
    std::process::exit(code.into());
}

#[async_std::main]
async fn main() {
    log::parse_log_level();

    let _b = Broker::new().await;

    let app = cfg::cli();
    let cfg = cfg::match_fold(app);
    let State { qaul, router: _ } = State::new(&cfg).await;

    let _rpc = RpcServer::start_default(qaul).await.unwrap();

    // // !no_upnp means upnp has _not_ been disabled
    // if !cfg.no_upnp {
    //     if upnp::open_port(cfg.port).is_none() {
    //         error!("Failed to open UPNP port; your router probably doesn't support it...");
    //     }
    // }

    // Never return the main thread
    let _: () = future::poll_fn(|_| Poll::Pending).await;
}
