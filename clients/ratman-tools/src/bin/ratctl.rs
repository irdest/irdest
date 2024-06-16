// SPDX-FileCopyrightText: 2019-20223 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use std::{collections::BTreeMap, env, fs::File, io::Read, path::PathBuf, sync::Arc};

use clap::{arg, Arg, ArgAction, ArgMatches, Command};
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    tokio::{self, fs::read_to_string},
    types::{AddrAuth, Address, Ident32},
    Result,
};
use serde::{Deserialize, Serialize};
// use libratman::client::{Address, RatmanIpc};

fn setup_cli() -> Command {
    Command::new("ratctl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Command line management interface for Ratman, the decentralised mesh router")
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
                arg!(-q --quiet "Disable additional output.  Results are still sent to stdout, making it easier to use ratctl in scripts")
            ]
        )
        .subcommands([
            Command::new("idpath"),
            //// \^-^/ Address management commands
            ////
            //// Addresses can be created and destroyed easily.
            ////
            //// Adding --force will delete data that is associated
            //// with the address and is still being referred to by
            //// other addresses (data loss).
            Command::new("addr")
                .about("Manage addresses")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("create")
                        .about("Create a new random address")
                        .args([
                            Arg::new("priv-name")
                                .long("name")
                                .help("A private identity name")
                                .action(ArgAction::Set),
                            Arg::new("namespace")
                                .long("space")
                                .help("A shared network namespace.  This API is a bit of a hack ;-;  \
                                       Instead of having ratmand generate a keypair and hand out the public address representation, \
                                       here you have to provide the private key information for the namespace.  \
                                       This means that the public address differs, and anyone else on the namespace needs to do the same.")
                                .action(ArgAction::Set)
                        ]),
                    Command::new("up")
                        .about("Mark the given address as online"),
                    Command::new("down")
                        .about("Mark the given address as offline"),
                    Command::new("destroy")
                        .about("Delete the given address, optionally with associated data")
                        .args([
                            Arg::new("force")
                                .help("Delete all data referred to by the deleted address")
                                .long("force")
                                .short('f')
                        ]),
                ]),
            //// \^-^/ Contact management commands
            ////
            //// The contact book contains "virtual data", meaning
            //// associations of metadata for network addresses.  To
            //// make it easier to remember your friends on the
            //// network you can store personal notes on addresses you
            //// encounter.
            // Command::new("contact")
            //     .about("Manage a private identity contact book, allowing for custom notes and tags")
            //     .arg_required_else_help(true)
            //     .subcommands([
            //         Command::new("add")
            //             .about("Add a new contact with optional filter data")
            //             .args([
            //                 Arg::new("address")
            //                     .help("peer address to save as contact")
            //                     .action(ArgAction::Set),
            //                 Arg::new("note")
            //                     .short('n')
            //                     .help("Add a searchable note")
            //                     .action(ArgAction::Set),
            //                 Arg::new("tags")
            //                     .short('t')
            //                     .help("Add a set of searchable tags in the format '<key>=<val>'")
            //                     .action(ArgAction::Append),
            //                 Arg::new("trust")
            //                     .short('u')
            //                     .help("Set a custom trust level from 1 to 7")
            //                     .action(ArgAction::Set)
            //             ]),
            //         Command::new("delete")
            //             .about("Delete existing contact entries via filters")
            //             .args([
            //                 Arg::new("address")
            //                     .help("peer address to save as contact")
            //                     .action(ArgAction::Set),
            //                 Arg::new("note")
            //                     .short('n')
            //                     .help("Add a searchable note")
            //                     .action(ArgAction::Set),
            //                 Arg::new("tags")
            //                     .short('t')
            //                     .help("Add a set of searchable tags in the format '<key>=<val>'")
            //                     .action(ArgAction::Append),
            //                 Arg::new("trust")
            //                     .short('u')
            //                     .help("Set a custom trust level from 1 to 7")
            //                     .action(ArgAction::Set)
            //             ])
            //     ]),

            //// =^-^= Stream subscriptions & more
            ////
            //// A subscription listens to all incoming messages for a given
            //// network flood namespace.
            ////
            //// Ratcat can be used to fetch data from a subscription, or
            //// integrate with xargs to periodically run some custom command on
            //// the received message.
            ////
            //// Subscriptions can have additional data, for example igonring
            //// messages when the address is offline, or setting a
            //// pre-determined timeout, after which the subscription will be
            //// deleted, along with any data that hadn't been fetched before.
            Command::new("stream")
                .about("Manage stream subscriptions")
                .arg_required_else_help(true)
                .subcommands([
                    //// Subscribe to incoming streams based on a recpiient
                    Command::new("sub")
                        .about("Add a new subscription for the current identity")
                        .arg_required_else_help(true)
                        .args([
                            Arg::new("address")
                                .long("addr")
                                .short('a')
                                .help("Subscribe to messages sent to your a local address")
                                .action(ArgAction::Set),
                            Arg::new("namespace")
                                .long("space")
                                .short('s')
                                .help("Subscribe to messages sent to a namespace address")
                        ]),
                    Command::new("list")
                        .alias("ls")
                        .about("List available subscriptions for the currently selected address"),
                    Command::new("unsub")
                        .alias("del")
                        .about("Delete an existing subscription"),
                    Command::new("resub")
                        .about("Restore an existing subscription")
                        .arg(
                            Arg::new("sub_id")
                                .help("Specify the subscription to restore")
                                .action(ArgAction::Set)
                        ),
                ]),
            //// Query various types of status output
            Command::new("status")
                .about("See component status")
                .arg_required_else_help(true)
                .subcommands([
                    //// Print information about how many threads Ratman is
                    //// running, how much memory is being used, how many
                    //// clients and links are connected, how many peers are
                    //// seen online, what the network global latency is
                    Command::new("system").about("Print the overall system status"),
                    //// Print individual address statistics, for example how
                    //// much traffic that address has produced in various
                    //// timescales, etc
                    Command::new("addr")
                        .about("Print individual address statistics")
                        .args([
                            Arg::new("irdest address")
                                .help("Provide the address to query")
                                .default_value("current identity address")
                                .action(ArgAction::Set)
                        ]),
                    //// Print statistics about the throughput and
                    //// connectivity of a specific link
                    Command::new("link")
                        .about("Print network link statistics")
                        .args([
                            Arg::new("link id")
                                .help("Provide the link to query")
                                .action(ArgAction::Set)
                        ]),
                ]),
            // Command::new("peer").about("List and query peers")
            //     .arg_required_else_help(true)
            //     .subcommands([
            //         Command::new("query")
            //             .about("Search for a specific peer via a query")
            //             .args([
            //                 Arg::new("by note")
            //                     .short('n')
            //                     .action(ArgAction::Set)
            //                     .help("String search of contact notes"),
            //                 Arg::new("by tag")
            //                     .short('t')
            //                     .action(ArgAction::Set)
            //                     .help("Filter by a matching tag in the format '<key>=<val>'"),
            //                 Arg::new("trust above")
            //                     .long("tr-above")
            //                     .action(ArgAction::Set)
            //                     .help("Filter by a trust level higher (or equal) to the provided"),
            //                 Arg::new("trust below")
            //                     .long("tr-below")
            //                     .action(ArgAction::Set)
            //                     .help("Filter by a trust level below the provided")
            //             ]),
            //         Command::new("list")
            //             .about("List and browse all available peers")
            //             .arg(
            //                 Arg::new("interactive")
            //                     .short('i')
            //                     .help("Capture the terminal and provide an interactive address browser")
            //                     .action(ArgAction::SetTrue)
            //             ),
            //     ]),
            // Command::new("link")
            //     .about("Manage network links (netmods)")
            //     .before_help("Adding new netmods during runtime is not yet implemented.")
            //     .arg_required_else_help(true)
            //     .subcommands([
            //         Command::new("list")
            //             .about("List currently available netmod links"),
            //         Command::new("up")
            //             .about("Mark the given link as online"),
            //         Command::new("down")
            //             .about("Mark the given link as offline"),
            //     ]),
        ])
        .after_help(
            "For more documentation, please consult the user manual at https://docs.irde.st/user/",
        )
}

// async fn connect_ipc(bind: &str) -> Result<RatmanIpc, Box<dyn std::error::Error>> {
//     eprint//         ArCommand::new("")g::with_name("NAMESPACE")subcommandsln!("Connecting to IPC backend...");
//     Ok(RatmanIpc::anonymous(bind).await?)
// }

// async fn get_peers(ipc: &RatmanIpc) -> Result<Vec<Address>, Box<dyn std::error::Error>> {
//     Ok(ipc.get_peers().await?)
// }

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

async fn run_command(
    ipc: Arc<RatmanIpc>,
    identity_data: Option<(Address, AddrAuth)>,
    output_format: &OutputFormat,
    subcommand: &str,
    args: &ArgMatches,
) -> Result<String> {
    match subcommand {
        "addr" => {
            if let Some((subsub, matches)) = args.subcommand() {
                match subsub {
                    "create" => {
                        let space_data = matches
                            .get_one("namespace")
                            .map(|s| Ident32::from_string(s));
                        let name = matches.get_one::<String>("priv-name");

                        let (addr, auth) = ipc.addr_create(name, space_data).await?;

                        match output_format {
                            OutputFormat::Json => {
                                return Ok(serde_json::to_string(
                                    &vec![("addr", addr.to_string()), ("auth", auth.to_string())]
                                        .into_iter()
                                        .collect::<BTreeMap<&'static str, String>>(),
                                )?);
                            }
                            OutputFormat::Lines => {
                                return Ok(format!("{}\n{}", addr, auth.to_string()));
                            }
                        }
                    }
                    "up" if identity_data.is_some() => {
                        let (addr, auth) = identity_data.unwrap();
                        ipc.addr_up(auth, addr).await?;
                        return Ok(reply_ok(&output_format));
                    }
                    "down" if identity_data.is_some() => {
                        let (addr, auth) = identity_data.unwrap();
                        ipc.addr_down(auth, addr).await?;
                        return Ok(reply_ok(&output_format));
                    }
                    "destroy" if identity_data.is_some() => {
                        let (addr, auth) = identity_data.unwrap();
                        ipc.addr_destroy(auth, addr, true).await?;
                        return Ok(reply_ok(&output_format));
                    }
                    _ => {}
                }
            }
        }
        "stream" => {
            if let Some((subsub, submatches)) = args.subcommand() {
                match subsub {
                    "sub" if identity_data.is_some() => {
                        let (addr, auth) = identity_data.unwrap();
                        let addr_ = submatches.get_one::<String>("address");
                        let space = submatches.get_one::<String>("namespace");

                        let mut subs_handle = match (addr_, space) {
                            (Some(addr_), None) => {
                                ipc.subs_create(
                                    auth,
                                    addr,
                                    libratman::types::Recipient::Address(Address::from_string(
                                        addr_,
                                    )),
                                )
                                .await?
                            }
                            (None, Some(space)) => {
                                ipc.subs_create(
                                    auth,
                                    addr,
                                    libratman::types::Recipient::Namespace(Address::from_string(
                                        space,
                                    )),
                                )
                                .await?
                            }
                            _ => {
                                libratman::elog(
                                    "must only provide either '--addr' or '--space'",
                                    2,
                                );
                            }
                        };

                        // Since this program is about to shut down, we must
                        // print enough information so the user can spawn their
                        // own subscriber.
                        let sub_socket = subs_handle.peer_info();
                        let sub_id = subs_handle.sub_id();
                        match output_format {
                            OutputFormat::Json => {
                                return Ok(format!(
                                    "{}",
                                    serde_json::to_string(
                                        &vec![
                                            ("sub_id", sub_id.to_string()),
                                            ("socket", sub_socket)
                                        ]
                                        .into_iter()
                                        .collect::<BTreeMap<&'static str, String>>(),
                                    )
                                    .unwrap()
                                ));
                            }
                            OutputFormat::Lines => {
                                return Ok(format!("{}\n{}", sub_id.to_string(), sub_socket));
                            }
                        }
                    }
                    "list" => {
                        let (addr, auth) = identity_data.unwrap();
                        let subs = ipc.subs_available(auth, addr).await?;
                        match output_format {
                            OutputFormat::Json => {
                                return Ok(format!(
                                    "{}",
                                    serde_json::to_string(
                                        &subs
                                            .into_iter()
                                            .map(|sub_id| sub_id.to_string())
                                            .collect::<Vec<String>>()
                                    )
                                    .unwrap()
                                ));
                            }
                            OutputFormat::Lines => {
                                return Ok(subs
                                    .into_iter()
                                    .map(|sub_id| sub_id.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n"));
                            }
                        }
                    }
                    "unsub" => {}
                    "resub" => {}
                    _ => {}
                }
            }
        }
        "status" => {}
        _ => {}
    }

    Ok("Nothing was done".into())
}

#[tokio::main]
async fn main() {
    let cli = setup_cli();
    let m = cli.get_matches();

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
        let mut f = File::open(identity_file_path.clone())?;
        let mut s = String::new();
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
        if cmd == "idpath" {
            println!("{identity_file_path}");
            return;
        }

        match run_command(ipc, identity_data.ok(), out_f, cmd, matches).await {
            Ok(output) => {
                println!("{output}");
            }
            Err(e) => {
                eprintln!("Failed: {e}");
            }
        }
    }

    // if let Err(e) = ratman_tools::command_filter(m).await {
    //     eprintln!("Operation failed, an Error occurred:\n{}", e);
    // }
}
