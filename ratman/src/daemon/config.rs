use async_std::io;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use tracing::warn;

/// Encode the current ratmand configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub netmod_inet_enabled: bool,
    pub netmod_lan_enabled: bool,
    pub netmod_inet_bind: String,
    pub netmod_lan_bind: String,
    pub api_socket_bind: String,
    pub accept_unknown_peers: bool,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
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
            netmod_inet_enabled: false,
            netmod_lan_enabled: false,
            netmod_inet_bind: String::from("[::]:9000"),
            netmod_lan_bind: String::from("9001"),
            api_socket_bind: String::from("127.0.0.1:9020"),
            accept_unknown_peers: true,
        }
    }
}
