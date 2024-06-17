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
                arg!(-q --quiet "Disable additional output.  Results are still sent to stdout, making it easier to use ratcat in scripts")
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
                        .required(true)
                        .help("Address a message stream to a single network participant")
                        .action(ArgAction::Append)
                        .conflicts_with_all(["to-space"]),
                    Arg::new("to-space")
                        .short('s')
                        .long("space")
                        .required(true)
                        .help("Address a message stream to a namespace address")
                        .action(ArgAction::Append)
                        .conflicts_with_all(["to-address"]),
                    Arg::new("stream-size")
                        .long("size")
                        .short('z')
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
                        .action(ArgAction::Append)
                        .help("Filter incoming message streams by the recipient address"),
                    Arg::new("count")
                        .help("Set the number of message streams you want to receive")
                        .value_parser(value_parser!(u64))
                        .default_value("1"),
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

//         "recv" if identity_data.is_ok() => {
//             let (addr, auth) = identity_data.ok().unwrap();
//             let to_addr = Ident32::from_string(
//                 &matches.get_one::<String>("to").unwrap().replace('"', ""),
//             );

//             if matches.get_one::<String>("one").is_some() {
//                 let (lh, mut read_stream) = ipc
//                     .recv_one(
//                         auth,
//                         addr,
//                         libratman::types::Recipient::Address(Address::from_string(
//                             &to_addr.to_string(),
//                         )),
//                     )
//                     .await?;

//                 let stream_dir = PathBuf::new()
//                     .join(env_xdg_data().unwrap())
//                     .join("share/ratcat/streams");

//                 create_dir_all(stream_dir.clone()).await?;

//                 let letterhead_buf = serde_json::to_string_pretty(&vec![
//                     ("from".to_string(), lh.from.to_string()),
//                     ("to".to_string(), lh.to.inner_address().to_string()),
//                     ("payload_length".to_string(), lh.stream_size.to_string()),
//                     (
//                         "auxiliary_data".to_string(),
//                         format!("{:?}", lh.auxiliary_data),
//                     ),
//                 ])
//                 .unwrap();

//                 let lh_id = lh.clone().digest();

//                 let mut f =
//                     File::create(stream_dir.join(format!("letterhead_{}.json", lh_id))).await?;
//                 f.write_all(letterhead_buf.as_bytes()).await?;

//                 let mut stdout = tokio::io::stdout();
//                 copy_buf(&mut BufReader::new(read_stream.as_reader()), &mut stdout).await?;
//             } else if matches.get_one::<String>("many").is_some() {
//                 todo!()
//                 // ipc.recv_many(auth, addr, to, num)
//             }
//         }
//         x => {
//             eprintln!(
//                 "Action '{}' requires authentication. Either use -i <id-file> or -p <profile>",
//                 x
//             );
//         }
//     }
