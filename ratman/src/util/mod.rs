// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

pub mod cli;
pub mod codes;
pub mod fork;

pub(crate) mod pidfile;
// pub(crate) mod upnp; // FIXME: this currently doesn't work

use crate::config::{ConfigTree, SubConfig};
use libratman::tokio::sync::mpsc::{Receiver, Sender};
use nix::unistd::Uid;

use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

/// A convevient Sender/Receiver pair for a type
pub(crate) type IoPair<T> = (Sender<T>, Receiver<T>);

/// Setup a very verbose output for test environments
pub fn setup_test_logging() {
    let cfg = ConfigTree::default_in_memory().patch("ratmand/verbosity", "trace");
    setup_logging(cfg.get_subtree("ratmand").as_ref().unwrap());
}

/// Setup default logging output with a configuration
pub fn setup_logging(ratmand_config: &SubConfig) {
    let lvl = ratmand_config
        .get_string_value("verbosity")
        .unwrap_or_else(|| "debug".into());
    let syslog = ratmand_config.get_bool_value("use_syslog").unwrap_or(false);

    let filter = if lvl.contains(|x| x == ',') {
        lvl.split(|x| x == ',')
            .fold(EnvFilter::default(), |filter, lvl| {
                filter.add_directive(lvl.parse().unwrap())
            })
    } else {
        EnvFilter::default().add_directive(match lvl.as_str() {
            "trace" => LevelFilter::TRACE.into(),
            "debug" => LevelFilter::DEBUG.into(),
            "info" => LevelFilter::INFO.into(),
            "warn" => LevelFilter::WARN.into(),
            "error" => LevelFilter::ERROR.into(),
            _ => unreachable!(),
        })
    }
    .add_directive("tokio=error".parse().unwrap())
    .add_directive("mio=error".parse().unwrap())
    .add_directive("polling=error".parse().unwrap())
    .add_directive("trust_dns_proto=error".parse().unwrap())
    .add_directive("trust_dns_resolver=warn".parse().unwrap());

    // Initialise the logger
    if syslog {
        let identity = std::ffi::CStr::from_bytes_with_nul(b"ratmand\0").unwrap();
        let facility = Default::default();
        let syslog =
            tracing_syslog::Syslog::new(identity, tracing_syslog::Options::LOG_PID, facility);
        fmt()
            .with_ansi(false)
            .with_env_filter(filter)
            .with_writer(syslog)
            .init();
    } else {
        #[cfg(not(feature = "android"))]
        fmt().with_env_filter(filter).init();
    }

    info!("Initialised logger: welcome to ratmand!");
    trace!(
        "ratmand launched by user {} with parameters: {:?}",
        Uid::current(),
        std::env::args()
    );
}
