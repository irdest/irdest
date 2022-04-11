//! Ratman daemon entrypoint

#[macro_use]
extern crate tracing;

pub(crate) use ratman::*;

use clap::{App, Arg, ArgMatches};
use netmod_inet::{Endpoint as Inet, Mode};
use netmod_lan::{default_iface, Endpoint as LanDiscovery};
use std::{fs::File, io::Read};
use ratman::daemon::config::Config;

pub fn build_cli() -> ArgMatches<'static> {
    App::new("ratmand")
        .about("Decentralised and delay tolerant peer-to-peer packet router.  Part of the Irdest project: https://irde.st")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("This is ALPHA level software and will include bugs and cause crashes.  If you encounter a reproducible issue, please report it in our issue tracker (https://git.irde.st/we/irdest) or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
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
                // .required_unless_one(&["PEERS", "PEER_FILE", "NO_INET"])
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
            Arg::with_name("INET_BIND")
                .takes_value(true)
                .long("inet")
                .help("Specify the inet-driver socket bind address.  Make sure this port is open in your firewall")
                .default_value("[::]:9000"),
        )
        .arg(
            Arg::with_name("NO_INET")
                .long("no-inet")
                .help("Disable the inet overlay driver")
        )
        .arg(
            Arg::with_name("DISCOVERY_PORT")
                .long("discovery-port")
                .takes_value(true)
                .default_value("9001")
                .help("Specify the port used for local peer discovery.  Make sure this port is open in your firewall.  WARNING: it's not recommended to change this unless you know this is what you want!")
        )
        .arg(
            Arg::with_name("DISCOVERY_IFACE")
                .takes_value(true)
                .long("discovery-iface")
                .help("Specify the interface on which to bind for local peer discovery.  If none is provided the default interface will be attempted to be determined")
        )
        .arg(
            Arg::with_name("NO_DISCOVERY")
                .long("no-discovery")
                .help("Disable the local multicast peer discovery mechanism")
        )
        .arg(
            Arg::with_name("PEERS")
                .long("peers")
                .short("p")
                .help("Specify a set of peers via the PEER SYNTAX: <netmod-id>#<address>:<port>[L].  Incompatible with `-f`. Valid netmod-ids are tcp. Example: tcp#10.0.0.10:9000L")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("PEER_FILE")
                .long("peer-file")
                .short("f")
                .help("Provide a set of initial peers to connect to.  Incompatible with `-p`")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("USE_UPNP")
                .long("upnp")
                .hidden(true)
                .help("Attempt to open the port used by the inet driver in your local gateway")
        )
        .arg(
            Arg::with_name("NO_WEBUI")
                .long("no-webui")
                .help("Stop ratmand from serving a webui on port 8090")
        )
        .get_matches()
}

// Ok(()) -> all good
// Err(_) -> emit warning but keep going
async fn setup_local_discovery(
    r: &Router,
    m: &ArgMatches<'_>,
    c: &Config,
) -> std::result::Result<(String, u16), String> {

    let iface = m.value_of("DISCOVERY_IFACE")
        .map(Into::into)
        .or_else(|| default_iface().map(|iface| {
            info!("Auto-selected interface '{}' for local peer discovery.  Is this wrong?  Pass --discovery-iface to ratmand instead!", iface);
            iface
        })).ok_or("failed to determine interface to bind on".to_string())?;

    let port = m
        .value_of("DISCOVERY_PORT")
        .unwrap_or(c.netmod_lan_bind.as_str())
        .parse()
        .map_err(|e| format!("failed to parse discovery port: {}", e))?;

    r.add_endpoint(LanDiscovery::spawn(&iface, port)).await;
    Ok((iface, port))
}

#[async_std::main]
async fn main() {

    let configuration = match daemon::config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load/write configuration: {}. Resuming with default values.", e);
            daemon::config::Config::new()
        }
    };

    let m = build_cli();
    let dynamic = m.is_present("ACCEPT_UNKNOWN_PEERS") || configuration.accept_unknown_peers;


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
        None if !dynamic => daemon::elog("Failed to initialise ratmand: missing peers data!", 2),
        None => vec![],
    };

    let r = Router::new();
    if !m.is_present("NO_INET") || configuration.netmod_inet_enabled {
        let tcp = match Inet::new(
            m.value_of("INET_BIND").unwrap_or(configuration.netmod_inet_bind.as_str()),
            "ratmand",
            if dynamic { Mode::Dynamic } else { Mode::Static },
        )
        .await
        {
            Ok(tcp) => {
                // Open the UPNP port if the user enabled this feature
                if m.is_present("USE_UPNP") {
                    if let Err(e) = daemon::upnp::open_port(tcp.port()) {
                        error!("UPNP setup failed: {}", e);
                    }
                }

                let peers: Vec<_> = peers.iter().map(|s| s.as_str()).collect();
                match daemon::attach_peers(&tcp, peers).await {
                    Ok(()) => tcp,
                    Err(e) => daemon::elog(format!("failed to parse peer data: {}", e), 1),
                }
            }
            Err(e) => daemon::elog(format!("failed to initialise TCP endpoint: {}", e), 1),
        };

        r.add_endpoint(tcp).await;
    }

    // If local-discovery is enabled
    if !m.is_present("NO_DISCOVERY") || configuration.netmod_lan_enabled {
        match setup_local_discovery(&r, &m, &configuration).await {
            Ok((iface, port)) => debug!(
                "Local peer discovery running on interface {}, port {}",
                iface, port
            ),
            Err(e) => warn!("Failed to setup local peer discovery: {}", e),
        }
    }

    // If webui is enabled
    if !m.is_present("NO_WEBUI") {
        match daemon::web::start(r.clone(), "127.0.0.1", 8090).await {
            Ok(_) => {}
            Err(e) => warn!("Failed to setup webui bind {:?}", e),
        }
    }

    let api_bind = match m.value_of("API_BIND").unwrap_or(configuration.api_socket_bind.as_str()).parse() {
        Ok(addr) => addr,
        Err(e) => daemon::elog(format!("Failed to parse API_BIND address: {}", e), 2),
    };
    if let Err(e) = daemon::run(r, api_bind).await {
        error!("Ratmand suffered fatal error: {}", e);
    }
}
