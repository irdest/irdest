// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
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

pub fn setup_cli() -> Command {
    Command::new("ratcat")
        .about("Handle sending and receiving streams for Ratman")
        .version(env!("CARGO_PKG_VERSION"))
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
                Arg::new("quiet")
                    .action(ArgAction::SetTrue)
                    .short('q')
                    .help("Disable additional output.  Results are still sent to stdout, making it easier to use ratcat in scripts")
            ]
        )
        .subcommands([
            Command::new("idpath").about("Print the currently selected identity"),
            Command::new("send")
                .alias("s")
                .alias("send-to")
                .about("Send messages across the network")
                .args([
                    // Arg::new("to-contact").conflicts_with_all(["to-address", "flood"]).,
                    Arg::new("to-address")
                        .short('a')
                        .long("addr")
                        // .required(true)
                        .help("Address a message stream to a single network participant")
                        .action(ArgAction::Append)
                        .conflicts_with_all(["to-space"]),
                    Arg::new("to-space")
                        .short('s')
                        .long("space")
                        // .required(true)
                        .help("Address a message stream to a namespace address")
                        .action(ArgAction::Append)
                        .conflicts_with_all(["to-address"]),
                    Arg::new("chunk-size")
                        .help("Optionally split the incoming stream into smaller chunks to reduce transmission latency.  A value of 0 disables stream chunk splitting")
                        .value_parser(value_parser!(u64))
                        .short('z')
                        .long("chunk-size")
                        .action(ArgAction::Set)
                ]),
            Command::new("recv")
                .about("Set your computer to receive files")
                .args([
                    Arg::new("streams-count")
                        .short('c')
                        .help("Set the number of message streams you want to receive.  \
                               Value of 0 will receive streams forever.
NOTE: ratcat will not terminate on its own and will have to be stopped externally.  \
                               The API socket remains unavailable during this time.  \
                               For longer receive sessions it's recommended you set up a subscription instead")
                        .value_parser(value_parser!(u64))
                        .default_value("1"),
                    Arg::new("to-address")
                        .action(ArgAction::Set)
                        .help("Filter incoming message streams by the recipient address"),
                ])
        ])
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
        let quiet = base_args.quiet;

        match run_program(m, base_args).await {
            Ok(()) => {
                if !quiet {
                    eprintln!("ratcat completed successfully uwu");
                }
                std::process::exit(0);
            }
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
