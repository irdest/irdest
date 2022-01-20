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

#[macro_use]
extern crate tracing;

mod config;
mod inlet;
mod io;
mod outlet;
mod proto;
mod server;

use clap::{App, Arg, ArgMatches};
use config::Config;
use directories::ProjectDirs;
use server::Server;
use std::path::PathBuf;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

fn setup_cli() -> ArgMatches {
    App::new("irdest-proxy")
        .version(env!("CARGO_PKG_VERSION"))
        .about("An IP traffic proxy for a Ratman network")
        .after_help("By default irdest-proxy(1) stores its configuration files in $XDG_CONFIG_HOME/irdest-proxy.  You can override this behaviour via the --cfg-dir parameter")
        .arg(
            Arg::new("VERBOSITY")
                .takes_value(true)
                .short('v')
                .long("verbosity")
                .help("Specify to which degree this service should log")
                .default_value("info")
                .possible_values(["trace", "debug", "info", "warn", "error"]),
        )
        .arg(
            Arg::new("CONFIG_DIR")
                .takes_value(true)
                .short('d')
                .long("cfg-dir")
                .help("Override the default configuration directory location.  The directory must contain a `routes.pm` and `self.json` file"),
        )
        .arg(
            Arg::new("API_BIND")
                .takes_value(true)
                .long("ipc")
                .help("Override the default bind address of the Ratman IPC socket"),
        )
        .get_matches()
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
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    info!("Initialised logger: welcome to irdest-proxy!");
}

#[async_std::main]
async fn main() {
    let m = setup_cli();

    let verbosity = m.value_of("VERBOSITY").unwrap();
    let cfg_dir = m.value_of("CONFIG_DIR");
    let bind = m.value_of("API_BIND");

    setup_logging(verbosity);

    let cfg_dir = cfg_dir
        .map(|dir| PathBuf::new().join(dir))
        .unwrap_or_else(|| {
            let dir = ProjectDirs::from("org", "irdest", "irdest-proxy")
                .expect("failed to determine project directories on this platform!")
                .config_dir()
                .to_path_buf();
            let _ = std::fs::create_dir_all(&dir);
            dir
        });

    let cfg = match Config::load(cfg_dir) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("failed to load configuration: {}", e);
            std::process::exit(2);
        }
    };

    Server::new(cfg).run(bind).await
}
