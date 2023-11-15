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

// mod config;
// mod inlet;
// mod io;
// mod outlet;
// mod proto;
// mod server;

// #[cfg(test)]
// mod test;

// pub use config::{parse_routes_file, Config, Routes};

// use directories::ProjectDirs;
// use server::Server;
// use std::path::PathBuf;
// use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

// pub fn setup_logging(lvl: &str) {
//     let filter = EnvFilter::default()
//         .add_directive(match lvl {
//             "trace" => LevelFilter::TRACE.into(),
//             "debug" => LevelFilter::DEBUG.into(),
//             "info" => LevelFilter::INFO.into(),
//             "warn" => LevelFilter::WARN.into(),
//             "error" => LevelFilter::ERROR.into(),
//             _ => unreachable!(),
//         })
//         .add_directive("async_std=error".parse().unwrap())
//         .add_directive("async_io=error".parse().unwrap())
//         .add_directive("polling=error".parse().unwrap())
//         .add_directive("mio=error".parse().unwrap());

//     // Initialise the logger
//     fmt().with_env_filter(filter).init();
//     info!("Initialised logger: welcome to irdest-proxy!");
// }

// pub fn get_config_path() -> PathBuf {
//     ProjectDirs::from("org", "irdest", "irdest-proxy")
//         .expect("failed to determine configuration directory on your platform")
//         .config_dir()
//         .to_path_buf()
// }

// pub async fn start_proxy(verbosity: &str, bind: Option<&str>, config: Config, routes: Routes) {
//     setup_logging(verbosity);
//     Server::new(config, routes).run(bind).await
// }
