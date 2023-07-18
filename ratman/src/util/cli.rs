use clap::{App, Arg, ArgMatches};

// FIXME: This file is hated by rustfmt and I don't understand why

/// Start Ratman in the current process/ thread
pub fn build_cli() -> ArgMatches<'static> {
    App::new("ratmand")
        .about("Decentralised and delay tolerant peer-to-peer packet router.  \
                Part of the Irdest project: https://irde.st")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("This software is in ALPHA and will include bugs and cause crashes!
                     If you encounter a reproducible issue, \
                     please report it in our issue tracker (https://git.irde.st/we/irdest) \
                     or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
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
            Arg::with_name("CONFIG")
                .takes_value(true)
                .short("c")
                .long("config")
                .help("Override the configuration path from $XDG_CONFIG_HOME/ratmand/ratmand.kdl")
        )
        .arg(
            Arg::with_name("DAEMONIZE")
                .long("daemonize")
                .help("Fork ratmand into the background and detach it from the current stdout/stderr/tty")
        )
        .get_matches()
}

// .arg(
//     Arg::with_name("ACCEPT_UNKNOWN_PEERS")
//         .long("accept-unknown-peers")
//         .short("d")
//         // .required_unless_one(&["PEERS", "PEER_FILE", "NO_INET"])
//         .help("Configure ratmand to peer with any incoming connection it may encounter")
// )
// .arg(
//     Arg::with_name("API_BIND")
//         .takes_value(true)
//         .long("bind")
//         .short("b")
//         .help("Specify the API socket bind address")
//         .default_value("127.0.0.1:9020"),
// )
// .arg(
//     Arg::with_name("INET_BIND")
//         .takes_value(true)
//         .long("inet")
//         .help("Specify the inet-driver socket bind address.  Make sure this port is open in your firewall")
//         .default_value("[::]:9000"),
// )
// .arg(
//     Arg::with_name("NO_INET")
//         .long("no-inet")
//         .help("Disable the inet overlay driver")
// )
// .arg(
//     Arg::with_name("DISCOVERY_PORT")
//         .long("discovery-port")
//         .takes_value(true)
//         .default_value("9001")
//         .help("Specify the port used for local peer discovery.  Make sure this port is open in your firewall.  WARNING: it's not recommended to change this unless you know this is what you want!")
// )
// .arg(
//     Arg::with_name("DISCOVERY_IFACE")
//         .takes_value(true)
//         .long("discovery-iface")
//         .help("Specify the interface on which to bind for local peer discovery.  If none is provided the default interface will be attempted to be determined")
// )
// .arg(
//     Arg::with_name("NO_DISCOVERY")
//         .long("no-discovery")
//         .help("Disable the local multicast peer discovery mechanism")
// )
// .arg(
//     {
//         let arg = Arg::with_name("NO_LORA")
//         .long("no-lora")
//             .help("Disable the lora modem driver");

//         #[cfg(not(feature = "lora"))]
//         let arg = arg.hidden(true);
//         arg
//     })
// .arg(
//     {
//         let arg = Arg::with_name("DATALINK_IFACE")
//             .takes_value(true)
//             .long("datalink-iface")
//             .help("Specify datalink interface.");

//         #[cfg(not(feature = "datalink"))]
//         let arg = arg.hidden(true);
//         arg
//     })
// .arg(
//     {
//         let arg = Arg::with_name("SSID")
//             .takes_value(true)
//             .long("ssid")
//             .help("Specify SSID for wireless interface");

//         #[cfg(not(feature = "datalink"))]
//         let arg = arg.hidden(true);
//         arg
//     }
// )
// .arg(
//     {
//         let arg = Arg::with_name("NO_DATALINK")
//             .long("no-datalink")
//             .help("Disables datalink driver");

//         #[cfg(not(feature = "datalink"))]
//         let arg = arg.hidden(true);
//         arg
//     }
//     )
// .arg(
//     Arg::with_name("PEERS")
//         .long("peers")
//         .short("p")
//         .help("Specify a set of peers via the PEER SYNTAX: <netmod-id>#<address>:<port>[L].  Incompatible with `-f`. Valid netmod-ids are tcp. Example: tcp#10.0.0.10:9000L")
//         .takes_value(true)
//         .multiple(true),
// )
// .arg(
//     Arg::with_name("PEER_FILE")
//         .long("peer-file")
//         .short("f")
//         .help("Provide a set of initial peers to connect to.  Incompatible with `-p`")
//         .takes_value(true)
// )
// .arg(
//     Arg::with_name("USE_UPNP")
//         .long("upnp")
//         .hidden(true)
//         .help("Attempt to open the port used by the inet driver in your local gateway")
// )
// .arg(
//     Arg::with_name("NO_DASHBOARD")
//         .long("no-dashboard")
//         .help("Stop ratmand from serving a dashboard on port 8090")
// )

// .arg(
//     Arg::with_name("PID_FILE")
//         .takes_value(true)
//         .long("pid-file")
//         .help("A file which the process PID is written into")
//         .default_value("/tmp/ratmand.pid"),
// )
