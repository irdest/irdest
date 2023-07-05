// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Lux <lux@lux.name>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Ratman daemon entrypoint

// #[macro_use]
// extern crate tracing;

// pub(crate) use ratman::{daemon::startup::*, *};

// fn main() {
//     let mut config = match daemon::config::Config::load() {
//         Ok(cfg) => cfg,
//         Err(e) => {
//             error!(
//                 "Failed to load/write configuration: {}. Resuming with default values.",
//                 e
//             );
//             daemon::config::Config::new()
//         }
//     };

//     let m = build_cli();
//     config.apply_arg_matches(m);

//     if config.daemonize {
//         if let Err(err) = sysv_daemonize_app(config) {
//             eprintln!("Ratmand suffered fatal error: {}", err);
//             std::process::exit(-1);
//         }
//     } else if let Err(()) = async_std::task::block_on(run_app(config)) {
//         std::process::exit(-1);
//     }
// }

use std::{fs::File, io::Write};
use ratmand::config::ConfigTree;

#[async_std::main]
async fn main() {
    // ratmand::start_with_configuration(Config::default()).await

    let config = ConfigTree::create_new_default();

    // println!("{:#?}", doc);
    // let mut f = File::create("ratmand.kdl").unwrap();
    // f.write_all(doc.to_string().as_bytes()).unwrap();
}
