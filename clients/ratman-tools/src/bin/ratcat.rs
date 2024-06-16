// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use clap::{arg, value_parser, Arg, ArgAction, Command};
use libratman::{
    api::{RatmanIpcExtV1, RatmanStreamExtV1},
    env_xdg_data,
    tokio::{
        self,
        fs::{create_dir_all, File},
        io::{copy_buf, AsyncWriteExt, BufReader},
    },
    types::{AddrAuth, Address, Ident32, LetterheadV1},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum OutputFormat {
    Lines,
    Json,
}

fn reply_ok(output_format: &OutputFormat) -> String {
    match output_format {
        OutputFormat::Json => serde_json::to_string(
            &vec![("ok", true)]
                .into_iter()
                .collect::<BTreeMap<&'static str, bool>>(),
        )
        .unwrap(),
        OutputFormat::Lines => {
            format!("ok")
        }
    }
}

pub fn build_cli() -> Command {
    Command::new("ratcat")
        .about("Client management program not unlike cat, but for ratman")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("By default ratcat stores local address information in ~/.config/ratcat/id")
        .max_term_width(110)
        .subcommand_required(true)
        .args(
            [
                Arg::new("api_bind")
                    .action(ArgAction::Set)
                    .help("Override the default client API socket address")
                    .short('b')
                    .long("bind")
                    .default_value("127.0.0.1:5852"),
                Arg::new("cid")
                    .action(ArgAction::Set)
                    .help("Specify the path for the current identity")
                    .short('i')
                    .long("cid")
                    .default_value("$XDG_CONFIG_HOME/ratcat/id"),
                Arg::new("profile")
                    .action(ArgAction::Set)
                    .help("Use a named address profile")
                    .short('p')
                    .long("prof")
                    .default_value("id"),
                Arg::new("output-format")
                    .action(ArgAction::Set)
                    .help("Specify the desired output format for commands")
                    .short('o')
                    .long("out")
                    .value_parser(["lines", "json"])
                    .default_value("lines"),
                arg!(-q --quiet "Disable additional output.  Results are still sent to stdout, making it easier to use ratcat in scripts")
            ]
        )
        .subcommands([
            Command::new("send")
                .about("Send messages across the network")
                .args([
                    // Arg::new("to-contact").conflicts_with_all(["to-address", "flood"]).,
                    Arg::new("to-address")
                        .short('a')
                        .long("addr")
                        .required(true)
                        .help("Address a message stream to a single network participant")
                        .conflicts_with_all(["flood"]),
                    Arg::new("flood")
                        .short('f')
                        .long("flood")
                        .required(true)
                        .help("Address a message stream to a network namespace address")
                        .conflicts_with_all(["to-address"]),
                    Arg::new("stream-size")
                        .required(true)
                        .value_parser(value_parser!(u64))
                        .help("Specify the maximum length of the sending message stream")
                        .action(ArgAction::Set),
                ]),
            Command::new("recv")
                .about("Set your computer to receive files")
                .args([
                    Arg::new("to-address")
                        .required(true)
                        .action(ArgAction::Set)
                        .help("Filter incoming message streams by the target recipient"),
                    Arg::new("one")
                        .help("Receive exactly one incoming message stream"),
                    Arg::new("many")
                        .action(ArgAction::Set)
                        .help("Receive some number of message streams")
                ])
        ])
}

#[libratman::tokio::main]
async fn main() -> libratman::Result<()> {
    let m = build_cli().get_matches();

    let ipc = match libratman::api::RatmanIpc::start(libratman::api::default_api_bind()).await {
        Ok(ipc) => ipc,
        Err(e) => {
            eprintln!("failed to setup ratmand IPC session: {e}");
            std::process::exit(2);
        }
    };

    let output_format: String = m
        .get_one("output-format")
        .unwrap_or(&"lines".to_owned())
        .to_string();

    let profile: &String = m.get_one("profile").unwrap();

    let identity_file_path: String = m
        .get_one::<String>("cid")
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            env::var("XDG_CONFIG_HOME")
                .map(|config_home| {
                    PathBuf::new()
                        .join(config_home)
                        .join("ratcat")
                        .join(&profile)
                })
                .expect("Must set XDG_CONFIG_HOME")
                .to_str()
                .unwrap()
                .to_string()
        });

    let out_f = match output_format.as_str() {
        "lines" => &OutputFormat::Lines,
        "json" => &OutputFormat::Json,
        _ => unreachable!(),
    };

    let identity_data = (|| -> Result<(Address, AddrAuth)> {
        let mut f = std::fs::File::open(identity_file_path.clone())?;
        let mut s = String::new();

        use std::io::Read;
        f.read_to_string(&mut s)?;

        match out_f {
            OutputFormat::Lines => {
                let mut lines = s.lines();
                Ok((
                    Address::from_string(&lines.next().unwrap().to_string()),
                    AddrAuth::from_string(&lines.next().unwrap().to_string()),
                ))
            }
            OutputFormat::Json => {
                let mut map: BTreeMap<String, String> = serde_json::from_str(s.as_str()).unwrap();
                Ok((
                    Address::from_string(&map.remove("addr").unwrap()),
                    AddrAuth::from_string(&map.remove("auth").unwrap()),
                ))
            }
        }
    })();

    if let Some((cmd, matches)) = m.subcommand() {
        match cmd {
            "idpath" => {
                println!("{identity_file_path}");
                return Ok(());
            }
            "send" if identity_data.is_ok() => {
                let (addr, auth) = identity_data.ok().unwrap();
                let mut stdin = tokio::io::stdin();

                if let Some(to_addr) = matches.get_one::<String>("to-address") {
                    let lheadv1 = LetterheadV1 {
                        from: addr,
                        to: libratman::types::Recipient::Address(Address::from_string(
                            &to_addr.to_string(),
                        )),
                        payload_length: *matches.get_one::<u64>("stream-size").unwrap(),
                        auxiliary_data: vec![],
                    };
                    ipc.send_to(auth, lheadv1, &mut stdin).await?;

                    println!("{}", reply_ok(out_f));
                } else if let Some(space_key) = matches.get_one::<String>("flood") {
                    let lheadv1 = LetterheadV1 {
                        from: addr,
                        to: libratman::types::Recipient::Namespace(Address::from_string(
                            &space_key.to_string(),
                        )),
                        payload_length: matches
                            .get_one::<String>("stream_size")
                            .unwrap()
                            .parse::<u64>()
                            .unwrap(),
                        auxiliary_data: vec![],
                    };
                    ipc.send_to(auth, lheadv1, &mut stdin).await?;
                    println!("{}", reply_ok(out_f));
                }
            }
            "recv" if identity_data.is_ok() => {
                let (addr, auth) = identity_data.ok().unwrap();
                let to_addr = Ident32::from_string(
                    &matches
                        .get_one::<String>("to-address")
                        .unwrap()
                        .replace('"', ""),
                );

                if matches.get_one::<String>("one").is_some() {
                    let (lh, mut read_stream) = ipc
                        .recv_one(
                            auth,
                            addr,
                            libratman::types::Recipient::Address(Address::from_string(
                                &to_addr.to_string(),
                            )),
                        )
                        .await?;

                    let stream_dir = PathBuf::new()
                        .join(env_xdg_data().unwrap())
                        .join("share/ratcat/streams");

                    create_dir_all(stream_dir.clone()).await?;

                    let letterhead_buf = serde_json::to_string_pretty(&vec![
                        ("from".to_string(), lh.from.to_string()),
                        ("to".to_string(), lh.to.inner_address().to_string()),
                        ("payload_length".to_string(), lh.payload_length.to_string()),
                        (
                            "auxiliary_data".to_string(),
                            format!("{:?}", lh.auxiliary_data),
                        ),
                    ])
                    .unwrap();

                    let lh_id = lh.clone().digest();

                    let mut f =
                        File::create(stream_dir.join(format!("letterhead_{}.json", lh_id))).await?;
                    f.write_all(letterhead_buf.as_bytes()).await?;

                    let mut stdout = tokio::io::stdout();
                    copy_buf(&mut BufReader::new(read_stream.as_reader()), &mut stdout).await?;
                } else if matches.get_one::<String>("many").is_some() {
                    todo!()
                    // ipc.recv_many(auth, addr, to, num)
                }
            }
            x => {
                eprintln!(
                    "Action '{}' requires authentication. Either use -i <id-file> or -p <profile>",
                    x
                );
            }
        }
    }

    Ok(())
}
