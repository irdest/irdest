use async_std::net::SocketAddr;
use ratman_client::Identity;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

/// Encode the current routing configuration
pub struct Config {
    /// The address of _this_ application
    pub addr: Identity,
    /// A map of IP spaces -> addresses
    pub map: BTreeMap<IpSpace, (InOrOut, Identity)>,
}

fn read_to_string(p: &PathBuf) -> io::Result<String> {
    let mut f = File::open(p)?;
    let mut string = String::new();
    f.read_to_string(&mut string)?;
    Ok(string)
}

fn parse_self_cfg(p: PathBuf) -> io::Result<Identity> {
    match read_to_string(&p) {
        Ok(cfg) => match serde_json::from_str::<BTreeMap<String, String>>(&cfg) {
            Ok(ref mut map) if map.contains_key("addr") => {
                Ok(Identity::from_string(&map.remove("addr").unwrap()))
            }
            _ => {
                info!(
                        "failed to parse self.json... assuming first install and generating new address!"
                    );
                Ok(Identity::random())
            }
        },
        Err(_) => {
            let mut cfg = File::create(&p)?;
            let id = Identity::random().to_string();
            let mut map = BTreeMap::<String, String>::new();
            map.insert("addr".into(), id);
            let buf = serde_json::to_string_pretty(&map).expect("woopsie");
            cfg.write_all(&buf.as_bytes())?;
            parse_self_cfg(p)
        }
    }
}

impl Config {
    pub fn load(dir: PathBuf) -> io::Result<Self> {
        let addr = parse_self_cfg(dir.join("self.json"))?;

        let friends = read_to_string(&dir.join("routes.pm"))?;
        let map = friends.lines().fold(BTreeMap::new(), |mut map, line| {
            match parse_line(line) {
                Some((key, val)) => {
                    map.insert(key, val);
                }
                None => {
                    eprintln!("failed to parse config line: {}", line);
                }
            }

            map
        });

        Ok(Self { addr, map })
    }
}

/// Represent some kind of IP space information
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum IpSpace {
    Single(SocketAddr),
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
fn parse_line(line: &str) -> Option<(IpSpace, (InOrOut, Identity))> {
    if line.contains("->") {
        parse_outgoing(line)
    } else if line.contains("<-") {
        parse_incoming(line)
    } else {
        warn!("Invalid peer-map line syntax: `{}`", line);
        None
    }
}

fn parse_outgoing(line: &str) -> Option<(IpSpace, (InOrOut, Identity))> {
    let split: Vec<_> = line.split("->").collect();
    let socket = IpSpace::Single(split.get(0)?.parse().ok()?);
    let id = Identity::from_bytes(split.get(1)?.as_bytes());
    Some((socket, (InOrOut::Out, id)))
}

fn parse_incoming(line: &str) -> Option<(IpSpace, (InOrOut, Identity))> {
    let split: Vec<_> = line.split("<-").collect();
    let socket = IpSpace::Single(split.get(0)?.parse().ok()?);
    let id = Identity::from_bytes(split.get(1)?.as_bytes());
    Some((socket, (InOrOut::In, id)))
}
