//! Module only loaded when Ratman is running as a daemon

mod parse;
mod peers;
mod state;
mod transform;

#[cfg(feature = "upnp")]
pub mod upnp;

use std::net::SocketAddr;

use crate::{Message, Recipient, Router};
use async_std::{net::TcpListener, task::spawn};
use state::{DaemonState, OnlineMap};
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};
use types::Result;

pub use peers::attach_peers;

pub fn elog<S: Into<String>>(msg: S, code: u16) -> ! {
    error!("{}", msg.into());
    std::process::exit(code.into());
}

pub fn setup_logging(lvl: &str) {
    let filter = EnvFilter::default()
        .add_directive(match lvl {
            "trace" => LevelFilter::TRACE.into(),
            "debug" => LevelFilter::DEBUG.into(),
            "info" => LevelFilter::INFO.into(),
            "warn" => LevelFilter::WARN.into(),
            "error" => LevelFilter::ERROR.into(),
            _ => unreachable!(),
        })
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("async_io=error".parse().unwrap())
        .add_directive("polling=error".parse().unwrap())
        .add_directive("trust_dns_proto=error".parse().unwrap())
        .add_directive("trust_dns_resolver=warn".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    info!("Initialised logger: welcome to ratmand!");
}

async fn run_relay(r: Router, online: OnlineMap) {
    loop {
        let Message {
            id,
            sender,
            recipient,
            payload,
            timesig,
            sign,
        } = r.next().await;
        debug!("Receiving message for {:?}", recipient);
        let recv = types::api::receive_default(types::message::received(
            id,
            sender,
            match recipient {
                Recipient::User(id) => Some(id),
                Recipient::Flood => None,
            },
            payload,
            format!("{:?}", timesig),
            sign,
        ));

        match recipient {
            Recipient::User(ref id) => {
                if let Some(Some(ref mut io)) = online.lock().await.get(id).map(Clone::clone) {
                    info!("Forwarding message to online client!");
                    if let Err(e) = parse::forward_recv(io.as_io(), recv).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
            Recipient::Flood => {
                for (_, ref mut io) in online.lock().await.iter_mut() {
                    if io.is_none() && continue {}
                    if let Err(e) =
                        parse::forward_recv(io.as_mut().unwrap().as_io(), recv.clone()).await
                    {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
        }
    }
}

/// Run the daemon!
pub async fn run(r: Router, addr: SocketAddr) -> Result<()> {
    info!("Listening for API connections on socket {:?}", addr);
    let listener = TcpListener::bind(addr).await?;
    let mut state = DaemonState::new(&listener, r.clone());
    let online = state.get_online().await;

    let relay = spawn(run_relay(r.clone(), online));

    while let Ok(io) = state.listen_for_connections().await {
        let io = match io {
            Some(io) => io,
            None => continue,
        };

        info!("Established new client connection");
        spawn(parse::parse_stream(r.clone(), io.clone()));
    }

    relay.cancel().await;
    Ok(())
}
