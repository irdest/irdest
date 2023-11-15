// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Irdest Proxy allows you to send IP data through an Irdest network
//!
//! It uses (at some point in the future) a few different mechanisms
//! to achieve this:
//!
//! - Simple socket + port mapped to a Ratman address
//!
//! - Tap device mapping a whole IP space to a Ratman address
//!
//! - Tap device mapping different IP spaces to different Ratman
//!   addresses, using dynamic route announcements (a la BGP).
//!
//! Currently ___0___ of these mechanisms are implemented :)

// use clap::{App, Arg, ArgMatches};
// use irdest_proxy::{parse_routes_file, Config};
// use std::path::PathBuf;

// fn setup_cli() -> ArgMatches {
//     App::new("irdest-proxy")
//         .version(env!("CARGO_PKG_VERSION"))
//         .about("A TCP proxy to tunnel connections through a Ratman network")
//         .after_help(r#"By default irdest-proxy(1) stores its configuration files in $XDG_CONFIG_HOME/irdest-proxy.  You can override this behaviour via the --cfg-dir parameter.

// Check the user manual for instructions on setting up this program."#)
//         .arg(
//             Arg::new("VERBOSITY")
//                 .takes_value(true)
//                 .short('v')
//                 .long("verbosity")
//                 .help("Specify to which degree this service should log")
//                 .default_value("info")
//                 .possible_values(["trace", "debug", "info", "warn", "error"]),
//         )
//         .arg(
//             Arg::new("CONFIG_DIR")
//                 .takes_value(true)
//                 .short('d')
//                 .long("cfg-dir")
//                 .help("Override the default configuration directory location.  The directory must contain a `routes.pm` file"),
//         )
//         .arg(
//             Arg::new("API_BIND")
//                 .takes_value(true)
//                 .long("ipc")
//                 .help("Override the default bind address of the Ratman IPC socket"),
//         )
//         .get_matches()
// }

#[async_std::main]
async fn main() {
    // let m = setup_cli();

    println!("Down for maintenance :(");

    // let cfg_dir = m
    //     .value_of("CONFIG_DIR")
    //     .map(|s| PathBuf::new().join(s))
    //     .unwrap_or_else(|| irdest_proxy::get_config_path());

    // let verbosity = m.value_of("VERBOSITY").unwrap();
    // let bind = m.value_of("API_BIND");

    // let routes = parse_routes_file(&cfg_dir);
    // let config = Config::load_and_update(&cfg_dir, &routes);

    // irdest_proxy::start_proxy(verbosity, bind, config, routes).await;
}
