// SPDX-FileCopyrightText: 2019-20223 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use libratman::{
    api::{default_api_bind, RatmanIpc, RatmanIpcExtV1},
    types::error::UserError,
    RatmanError, Result,
};
use ratman_tools::{
    base_args::{parse_base_args, BaseArgs},
    command_filter, tokio_runtime,
};
use std::{env, net::SocketAddr, str::FromStr};

fn setup_cli() -> Command {
    Command::new("ratctl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Ratman management CLI for addresses, stream subscriptions, and more")
        .after_help("For more documentation, please consult the user manual at https://docs.irde.st/user/")
        .max_term_width(110)
        .subcommand_required(true)
        .args(
            [
                Arg::new("api-bind")
                    .action(ArgAction::Set)
                    .help("Override the default client API socket address")
                    .short('b')
                    .long("bind")
                    .default_value("127.0.0.1:5852"),
                Arg::new("curr-id")
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
            Command::new("idpath").about("Print the currently selected identity"),
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
                    Command::new("list")
                        .about("List available local addresses")
                ]),
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
                        .arg(Arg::new("sub_id")
                             .help("Provide the subscription ID to unsubscribe from")
                             .action(ArgAction::Set))
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
                ]),
            Command::new("peers")
                .about("Information about other peers on the network")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("list").about("List all available peers on the network along some metadata about them"),
                ]),
            //// Namespace management commands
            Command::new("space")
                .about("Manage shared address namespaces")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("register")
                        .about("Register a new namespace key which can be included in a third-party application")
                        .args([
                            Arg::new("file_name")
                                .help("Specify the output file name for the namespace key")
                                .short('f')
                                .required(true)
                                .action(ArgAction::Set)
                        ]),
                    Command::new("up")
                        .about("Mark a given namespace as 'up', enabling the router to respond to anycast pings and other protocols")
                        .args([
                            Arg::new("file_name")
                                .help("Specify the key file created by 'register'")
                                .short('f')
                                .action(ArgAction::Set)
                        ]),
                    Command::new("down")
                        .about("Mark a given namespace as 'down'")
                        .args([
                            Arg::new("file_name")
                                .help("Specify the key file created by 'register'")
                                .short('f')
                                .action(ArgAction::Set)
                        ]),
                    Command::new("anycast")
                        .about("Send an anycast probe to this namespace, returning address responses ordered by time")
                        .args([
                            Arg::new("timeout")
                                .help("Specify a timeout in milliseconds")
                                .short('t')
                                .value_parser(value_parser!(u64))
                                .action(ArgAction::Set)
                        ])
                ]),
        ])
        .after_help(
            "For more documentation, please consult the user manual at https://docs.irde.st/user/",
        )
}

async fn run_program(m: ArgMatches, base_args: BaseArgs) -> Result<()> {
    let api_bind = m.get_one::<String>("api-bind").map(|provided| {
        SocketAddr::from_str(provided.as_str()).map_err(|parse_err| {
            RatmanError::User(UserError::InvalidInput(
                format!("Provided socket address could not be parsed: {}", parse_err),
                None,
            ))
        })
    });

    let ratmand_socket = match api_bind {
        Some(socket) => socket?,
        None => {
            eprintln!("Selected default socket location: {}", default_api_bind());
            default_api_bind()
        }
    };

    let ipc = RatmanIpc::start(ratmand_socket).await?;

    command_filter(&ipc, base_args, m).await
}

fn main() {
    let r = tokio_runtime();

    r.block_on(async {
        let cli = setup_cli();
        let m = cli.get_matches();
        let base_args = parse_base_args(&m);

        match run_program(m, base_args).await {
            Ok(()) => std::process::exit(0),

            Err(RatmanError::User(u)) => {
                eprintln!("You did it wrong: {u}");
                std::process::exit(1);
            }
            Err(RatmanError::ClientApi(c)) => {
                eprintln!("Client-Router communication error: {c}");
                std::process::exit(2);
            }
            Err(e) => {
                eprintln!("ratcat encountered an error: {e}");
                std::process::exit(2);
            }
        }
    });
}
