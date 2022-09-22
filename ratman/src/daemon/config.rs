use async_std::io;
use clap::ArgMatches;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use tracing::warn;

/// Encode the current ratmand configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub verbosity: String,
    pub accept_unknown_peers: bool,
    pub api_bind: String,

    pub inet_bind: String,
    pub no_inet: bool,

    pub discovery_port: u16,
    pub discovery_iface: Option<String>,
    pub no_discovery: bool,

    pub lora_port: Option<String>,
    pub lora_baud: u32,
    pub no_lora: bool,

    pub peers: Option<String>,
    pub peer_file: Option<String>,
    pub no_peering: bool,

    pub use_upnp: bool,
    pub no_dashboard: bool,
    pub daemonize: bool,
    pub pid_file: String,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take an existing configuration (for example one loaded from
    /// disk) and apply the CLI overrides to it
    pub fn apply_arg_matches(&mut self, m: ArgMatches<'static>) {
        if let Some(verbosity) = m.value_of("VERBOSITY") {
            self.verbosity = verbosity.into();
        }

        self.accept_unknown_peers |= m.is_present("ACCEPT_UNKNOWN_PEERS");

        if let Some(api_bind) = m.value_of("API_BIND") {
            self.api_bind = api_bind.into();
        }

        if let Some(inet_bind) = m.value_of("INET_BIND") {
            self.inet_bind = inet_bind.into();
        }

        self.no_inet |= m.is_present("NO_INET");

        if let Some(discovery_port) = m.value_of("DISCOVERY_PORT") {
            self.discovery_port = discovery_port.parse().unwrap();
        }

        if let Some(discovery_iface) = m.value_of("DISCOVERY_IFACE") {
            self.discovery_iface = Some(discovery_iface.into());
        }

        self.no_discovery |= m.is_present("NO_DISCOVERY");

        if let Some(peers) = m.value_of("PEERS") {
            self.peers = Some(peers.into());
        }

        if let Some(peer_file) = m.value_of("PEER_FILE") {
            self.peer_file = Some(peer_file.into());
        }

        self.no_dashboard |= m.is_present("NO_DASHBOARD");
    }

    pub fn load() -> std::io::Result<Config> {
        if let Some(dirs) = ProjectDirs::from("org", "irdest", "ratmand") {
            let config_path = dirs.config_dir().join("config.json");
            if config_path.exists() {
                // 1. Read configuration file
                // 2. Return Config struct

                let mut config_buffer = String::new();
                File::open(config_path)?.read_to_string(&mut config_buffer)?;

                let config: Config = serde_json::from_str(config_buffer.as_str())?;
                Ok(config)
            } else {
                // 1. Create configuration file with defaults
                // 2. Return Config struct

                let config = Config::default();
                let config_dir_path = config_path
                    .parent()
                    .ok_or(io::Error::new(ErrorKind::NotFound, "Directory not found."))?;
                std::fs::create_dir_all(config_dir_path)?;

                let mut config_file = File::create(config_path)?;
                let config_buffer = serde_json::to_string_pretty(&config)?;
                config_file.write(config_buffer.as_bytes())?;
                Ok(config)
            }
        } else {
            warn!("Failed to determine ProjectDirs on machine.");
            Ok(Config::default())
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            verbosity: "debug".into(),
            accept_unknown_peers: false,
            api_bind: "127.0.0.1:9020".into(),

            inet_bind: "[::]:9000".into(),
            no_inet: false,

            discovery_port: 9001,
            discovery_iface: None,
            no_discovery: false,

            lora_port: None,
            lora_baud: 9600,
            no_lora: true,

            peers: None,
            peer_file: None,
            no_peering: false,

            use_upnp: false,
            no_dashboard: false,
            daemonize: false,
            pid_file: "/tmp/ratmand.pid".into(),
        }
    }
}
