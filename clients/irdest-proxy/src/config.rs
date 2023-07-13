// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::net::SocketAddr;
use libratman::types::Address;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub type Routes = BTreeMap<IpSpace, (InOrOut, Address)>;

/// Encode the current routing configuration
///
/// The `addresses` field is only relevant for Inlets and is
/// automatically populated.  Each "bind" is registered as a new
/// address.  These addresses are then be re-used between runs.
#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// Inlet addresses per-route
    pub addresses: BTreeMap<String, Address>,
}

impl Config {
    pub fn get_address(&self, ip: &IpSpace) -> Address {
        *self
            .addresses
            .get(&ip.to_string())
            .expect("failed to load address from config")
    }

    /// Load the current configuration (creating it if none exists)
    /// and generating new addresses for any additional Inlet route
    /// that exists.
    pub fn load_and_update(dir: &PathBuf, routes: &Routes) -> Self {
        let path = dir.join("config.json");

        let mut f = File::open(&path)
            .unwrap_or_else(|_| File::create(&path).expect("failed to create config file"));

        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .expect("failed to read config file");

        let mut cfg = serde_json::from_str(&buf).unwrap_or_else(|_| Config::default());

        for (ip, (io, _)) in routes {
            // We only care about Inlet routes
            if io == &InOrOut::Out {
                continue;
            }

            // If this inlet route is new we generate a unique address for it
            let ip_str = ip.to_string();
            if !cfg.addresses.contains_key(&ip_str) {
                cfg.addresses.insert(ip_str, Address::random());
            }
        }

        // After generating new addresses for inlet routes we save
        // this to the configuration
        f.write_all(&serde_json::to_string_pretty(&cfg).unwrap().as_bytes())
            .unwrap();

        cfg
    }
}

pub fn parse_routes_file(dir: &PathBuf) -> Routes {
    let path = dir.join("routes.pm");

    let mut f = File::open(&path)
        .expect("Couldn't find routes.pm in your config directory.  This file is required!");
    let mut friends = String::new();
    f.read_to_string(&mut friends).unwrap();

    friends.lines().fold(Routes::new(), |mut map, line| {
        match parse_line(line) {
            Some((key, val)) => {
                map.insert(key, val);
            }
            None => {
                eprintln!("failed to parse config line: {}", line);
            }
        }

        map
    })
}

/// Represent some kind of IP space information
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum IpSpace {
    Single(SocketAddr),
}

impl ToString for IpSpace {
    fn to_string(&self) -> String {
        match self {
            Self::Single(addr) => addr.to_string(),
        }
    }
}

impl IpSpace {
    pub fn socket_addr(&self) -> &SocketAddr {
        match self {
            Self::Single(ref addr) => addr,
        }
    }
}

/// An enum that's either `In` or `Out`
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum InOrOut {
    In,
    Out,
}

/// Parse a single line of configuration into a routing tuple
fn parse_line(line: &str) -> Option<(IpSpace, (InOrOut, Address))> {
    if line.contains("<-") {
        parse_outgoing(line)
    } else if line.contains("->") {
        parse_incoming(line)
    } else {
        warn!("Invalid peer-map line syntax: `{}`", line);
        None
    }
}

fn parse_outgoing(line: &str) -> Option<(IpSpace, (InOrOut, Address))> {
    let split: Vec<_> = line.split("<-").collect();
    let socket = IpSpace::Single(split.get(0)?.trim().parse().ok()?);
    let id = Address::from_string(&split.get(1)?.trim().to_string());
    Some((socket, (InOrOut::Out, id)))
}

fn parse_incoming(line: &str) -> Option<(IpSpace, (InOrOut, Address))> {
    let split: Vec<_> = line.split("->").collect();
    let socket = IpSpace::Single(split.get(0)?.trim().parse().ok()?);
    let id = Address::from_string(&split.get(1)?.trim().to_string());
    Some((socket, (InOrOut::In, id)))
}

#[test]
fn test_parse_line_out() {
    let line = "127.0.0.1:443 <- 7053-2C1D-15D9-4D30-4FC5-4663-28BD-2E0C-F33D-0D49-2E28-6C1F-5649-6922-7DA8-B7A5";
    use std::net::*;

    match parse_line(line) {
        Some((ip, (io, id))) => {
            assert_eq!(
                IpSpace::Single(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    443
                ))),
                ip
            );

            assert_eq!(io, InOrOut::Out);
            assert_eq!(id, Address::from_string(&"7053-2C1D-15D9-4D30-4FC5-4663-28BD-2E0C-F33D-0D49-2E28-6C1F-5649-6922-7DA8-B7A5".to_owned()))
        }
        _ => panic!("invalid parse"),
    }
}

#[test]
fn test_parse_line_in() {
    let line = "0.0.0.0:8000 -> 7053-2C1D-15D9-4D30-4FC5-4663-28BD-2E0C-F33D-0D49-2E28-6C1F-5649-6922-7DA8-B7A5";
    use std::net::*;

    match parse_line(line) {
        Some((ip, (io, id))) => {
            assert_eq!(
                IpSpace::Single(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(0, 0, 0, 0),
                    8000
                ))),
                ip
            );

            assert_eq!(io, InOrOut::In);
            assert_eq!(id, Address::from_string(&"7053-2C1D-15D9-4D30-4FC5-4663-28BD-2E0C-F33D-0D49-2E28-6C1F-5649-6922-7DA8-B7A5".to_owned()))
        }
        _ => panic!("invalid parse"),
    }
}
