// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::{
    fs::{create_dir_all, File},
    io::{stdin, stdout, ReadExt, WriteExt},
};
use clap::{App, Arg};
use directories::ProjectDirs;
use libratman::client::{Address, Id, RatmanIpc, Receive_Type};
use serde::{Deserialize, Serialize};
use std::{os::unix::prelude::AsRawFd, path::PathBuf};

pub(crate) fn env_xdg_config() -> Option<String> {
    std::env::var("XDG_CONFIG_HOME").ok()
}

pub fn build_cli() -> App<'static, 'static> {
    App::new("ratcat")
        .about("Client management program not unlike cat, but for ratman")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("ratcat(1) stores current address information in $XDG_CONFIG_DIR/ratcat/config\n\nThis is ALPHA level software and will include bugs and cause crashes.  If you encounter a reproducible issue, please report it in our issue tracker (https://git.irde.st/we/irdest) or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
        .max_term_width(120)
        .arg(
            Arg::with_name("RECIPIENT")
                .takes_value(true)
                .required_unless_one(&["REGISTER", "RECEIVE"])
                .help("Specify the message recipient address.  Not required when calling `--register` or `--recv`")
        )
        .arg(
            Arg::with_name("MESSAGE")
                .takes_value(true)
                .help("Provide a message to send across the network.  If no <MESSAGE> is provided ratcat will read a message from standard input!")
        )
        .arg(
            Arg::with_name("SENDER")
                .long("sender")
                .short("s")
                .takes_value(true)
                .help("Specify the sender address instead of using the default one stored in $XDG_CONFIG_HOME")
        )
        .arg(
            Arg::with_name("REGISTER")
                .long("register")
                .help("Register a new address on the network with the Ratman daemon")
        )
        .arg(
            Arg::with_name("NO_DEFAULT")
                .long("no-default")
                .requires("REGISTER")
                .help("Don't use the new address as a default for ratcat in the future")
        )
        .arg(
            Arg::with_name("RECEIVE")
                .long("recv")
                .help("Set your computer to receive data via ratcat.")
        )
        .arg(
            Arg::with_name("RECV_COUNT")
                .long("count")
                .takes_value(true)
                .help("Specify the number of messages that `--recv` should wait for.  Default value is to wait forever.")
        )
        .arg(
            Arg::with_name("API_BIND")
                .takes_value(true)
                .long("bind")
                .short("b")
                .help("Specify the API socket bind address")
                .default_value("127.0.0.1:5852"),
        )
        .arg(
            Arg::with_name("CONFG")
                .takes_value(true)
                .long("config")
                .help("Override the position where ratcat should store its state configuration")
        )
}

#[derive(Serialize, Deserialize)]
struct Config {
    addr: Address,
    token: Id,
}

async fn register(
    path: PathBuf,
    bind: &str,
    no_default: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let stdout = stdout();
    let is_tty = nix::unistd::isatty(stdout.as_raw_fd()).unwrap_or(false);

    let ipc = connect_ipc(None, None, bind).await?;

    // For humans we print a friendly message
    if is_tty {
        eprintln!("Registered address: {}", ipc.address());
    }
    // For machines we get straight to the point
    else {
        println!("{}", ipc.address());
    }

    if !no_default {
        let mut f = File::create(path.join("config")).await?;

        let cfg = Config {
            addr: ipc.address(),
            token: ipc.token(),
        };
        let cfg_str = serde_json::to_string_pretty(&cfg)?;
        f.write_all(cfg_str.as_bytes()).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn connect_ipc(
    addr: Option<Address>,
    token: Option<Id>,
    bind: &str,
) -> Result<RatmanIpc, Box<dyn std::error::Error>> {
    let mut res = Err(libratman::ClientError::ConnectionLost.into());

    for _ in 0..3 {
        res = RatmanIpc::connect(bind, addr, token).await;
        if res.is_ok() {
            break;
        }

        eprintln!("ratmand connection failed: retrying in 750ms...");
        async_std::task::sleep(std::time::Duration::from_millis(750)).await;
    }

    Ok(res?)
}

/// Returns the number of messages sent
async fn send(
    ipc: &RatmanIpc,
    recp: &str,
    msg: Option<&str>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let recp = Address::from_string(&recp.to_string());

    // Either turn the provided string message into a byte array or read from stdin
    match msg.map(|s| s.as_bytes().to_vec()) {
        Some(msg) => {
            ipc.send_to(recp, msg).await?;
            Ok(1)
        }
        None => {
            let mut ctr = 0;
            let stdin = stdin();
            let mut buf = String::new();
            while let Ok(num_read) = stdin.read_line(&mut buf).await {
                if num_read == 0 {
                    break;
                }

                ipc.send_to(recp, buf.trim().as_bytes().to_vec()).await?;
                ctr += 1;
            }
            Ok(ctr)
        }
    }
}

async fn handle_receives(ipc: &RatmanIpc, num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let is_tty = nix::unistd::isatty(stdout.as_raw_fd()).unwrap_or(false);

    for _ in if num == 0 { 0..std::usize::MAX } else { 0..num } {
        let (tt, msg) = match ipc.next().await {
            Some(msg) => msg,
            None => break,
        };

        if tt == Receive_Type::FLOOD {
            continue;
        }

        let payload: Vec<_> = msg.get_payload();
        if is_tty {
            println!(
                "{}",
                String::from_utf8(payload).unwrap_or("<Unprintable data>".to_string())
            );
        } else {
            stdout.write_all(&payload).await?;
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() {
    let app = build_cli();
    let m = app.clone().get_matches();

    //// Setup the config location
    ////
    //// 1) If it is provided, use the command-line version
    //// 2) Otherwise use the environment variable
    //// 3) OTHERWISE use the XDG_CONFIG_HOME location
    let cfg_dir = m
        .value_of("CONFIG")
        .map(|path| PathBuf::new().join(path))
        .unwrap_or_else(|| {
            env_xdg_config()
                .map(|path| PathBuf::new().join(path))
                .unwrap_or_else(|| {
                    let dirs = ProjectDirs::from("org", "irdest", "ratcat")
                        .expect("Failed to initialise project directories for this platform!");
                    dirs.config_dir().to_path_buf()
                })
        });
    let _ = create_dir_all(&cfg_dir).await;

    let num: usize = match m.value_of("RECV_COUNT").map(|c| c.parse().ok()) {
        Some(Some(num)) => num,
        Some(None) => {
            eprintln!("Failed to parse `--count` as a number!");
            std::process::exit(2);
        }
        None => 0,
    };

    //// To register is a bit special because we terminate afterwards
    let api_addr = m.value_of("API_BIND").unwrap();
    if m.is_present("REGISTER") {
        match register(cfg_dir, api_addr, m.is_present("NO_DEFAULT")).await {
            Ok(true) => {
                eprintln!("You may now run `ratcat` to send data");
                std::process::exit(0);
            }
            Ok(false) => {
                eprintln!("Ratcat will not use this address for the future (because --no-default was provided)");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("An error occured during registration: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    //// Open the configuration a previous us left behind :)
    let mut cfg = match File::open(cfg_dir.join("config")).await {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s).await.unwrap();
            serde_json::from_str::<Config>(s.as_str()).expect("failed to parse config!")
        }
        Err(_) => {
            eprintln!("No configuration found!  Run `ratcat --register` first!");
            std::process::exit(2);
        }
    };

    //// Check if a sender address was provided via CLI options
    if let Some(addr) = m.value_of("SENDER") {
        cfg.addr = Address::from_string(&addr.to_owned());
    }

    //// We always need to connect to the IPC backend with our address
    eprintln!("Connecting to IPC backend...");
    let ipc = match connect_ipc(Some(cfg.addr), Some(cfg.token), api_addr).await {
        Ok(ipc) => ipc,
        Err(e) => {
            eprintln!("Failed to connect to Ratman daemon: {}", e);
            std::process::exit(1);
        }
    };

    //// If we were given a recipient we send try to send some data
    if let Some(recipient) = m.value_of("RECIPIENT") {
        let message = m.value_of("MESSAGE");
        match send(&ipc, recipient, message).await {
            Ok(num) => num,
            Err(e) => {
                eprintln!("An error occured during sending: {:?}", e);
                std::process::exit(1);
            }
        };
    }

    //// If we were given the --recv flag we try to receive some data
    if m.is_present("RECEIVE") {
        match handle_receives(&ipc, num).await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                std::process::exit(1);
            }
        }
    }
}
