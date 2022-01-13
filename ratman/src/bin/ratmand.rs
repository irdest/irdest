//! Ratman daemon entrypoint

#[macro_use]
extern crate tracing;

pub(crate) use ratman::*;

use clap::{App, Arg, ArgMatches};
use netmod_tcp::{Endpoint as TcpEp, Mode};
use std::{fs::File, io::Read};

pub fn build_cli() -> ArgMatches<'static> {
    App::new("ratmand")
        .about("Decentralised and delay tolerant peer-to-peer packet router.  Part of the Irdest project: https://irde.st")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("This is ALPHA level software and will include bugs and cause crashes.  If you encounter a reproducable issue, please report it in our issue tracker (https://git.irde.st/we/irdest) or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
        .max_term_width(120)
        .arg(
            Arg::with_name("VERBOSITY")
                .takes_value(true)
                .short("v")
                .long("verbosity")
                .possible_values(&["trace", "debug", "info", "warn", "error", "fatal"])
                .default_value("info")
                .help("Specify the verbosity level at which ratmand logs interactions"),
        )
        .arg(
            Arg::with_name("ACCEPT_UNKNOWN_PEERS")
                .long("accept-unknown-peers")
                .short("d")
                .required_unless_one(&["PEERS", "PEER_FILE"])
                .help("Configure ratmand to peer with any incoming connection it may encounter")
        )
        .arg(
            Arg::with_name("API_BIND")
                .takes_value(true)
                .long("bind")
                .short("b")
                .help("Specify the API socket bind address")
                .default_value("127.0.0.1:9020"),
        )
        .arg(
            Arg::with_name("TCP_BIND")
                .takes_value(true)
                .long("tcp")
                .help("Specify the tcp socket bind address")
                .default_value("[::]:9000"),
        )
        .arg(
            Arg::with_name("UDP_BIND")
                .takes_value(true)
                .long("udp")
                .help("Specify the udp socket bind address")
                .default_value("[::]:9001"),
        )
        .arg(
            Arg::with_name("PEERS")
                .long("peers")
                .short("p")
                .help("Specify a set of peers via the PEER SYNTAX: <netmod-id>#<address>[:<port>].  Incompatible with `-f`/ `-peer-file`")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("PEER_FILE")
                .long("peer-file")
                .short("f")
                .help("Provide a set of initial peers to connect to.  Incompatible with `-p`/ `-peers`")
                .takes_value(true)
        )
        .get_matches()
}

#[async_std::main]
async fn main() {
    let m = build_cli();

    // Setup logging
    daemon::setup_logging(m.value_of("VERBOSITY").unwrap());

    // Load peers or throw an error about missing cli data!
    let peers: Vec<_> = match m
        .value_of("PEERS")
        .map(|s| s.replace(" ", "\n").to_owned())
        .or(m.value_of("PEER_FILE").and_then(|path| {
            let mut f = File::open(path).ok()?;
            let mut buf = String::new();
            f.read_to_string(&mut buf).ok()?;
            Some(buf)
        }))
        .or(if m.is_present("NO_PEERING") {
            Some("".into())
        } else {
            None
        }) {
        Some(peer_str) => peer_str.split("\n").map(|s| s.trim().to_owned()).collect(),
        None => daemon::elog("Failed to initialise ratmand: missing peers data!", 2),
    };

    // Setup the Endpoints
    let tcp = match TcpEp::new(
        m.value_of("TCP_BIND").unwrap(),
        "ratmand",
        if m.is_present("ACCEPT_UNKNOWN_PEERS") {
            Mode::Dynamic
        } else {
            Mode::Static
        },
    )
    .await
    {
        Ok(tcp) => {
            let peers = peers.iter().map(|s| s.as_str()).collect();
            match daemon::attach_peers(&tcp, peers).await {
                Ok(()) => tcp,
                Err(e) => daemon::elog(format!("failed to parse peer data: {}", e), 1),
            }
        }
        Err(e) => daemon::elog(format!("failed to initialise TCP endpoint: {}", e), 1),
    };

    let r = Router::new();
    r.add_endpoint(tcp).await;

    let api_bind = match m.value_of("API_BIND").unwrap().parse() {
        Ok(addr) => addr,
        Err(e) => daemon::elog(format!("Failed to parse API_BIND address: {}", e), 2),
    };
    if let Err(e) = daemon::run(r, api_bind).await {
        error!("Ratmand suffered fatal error: {}", e);
    }
}
