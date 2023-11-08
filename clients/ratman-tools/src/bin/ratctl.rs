// SPDX-FileCopyrightText: 2019-20223 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use clap::{arg, Arg, ArgAction, Command};
// use libratman::client::{Address, RatmanIpc};

fn setup_cli() -> Command {
    Command::new("ratctl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Command line management interface for Ratman, the decentralised mesh router")
        .max_term_width(110)
        .arg_required_else_help(true)
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
                arg!(-q --quiet "Disable additional output.  Results are still sent to stdout, making it easier to use ratctl in scripts")
            ]
        )
        .subcommands([
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
                            Arg::new("name")
                                .help("A private identity name")
                                .required(true)
                                .action(ArgAction::Set)
                        ]),
                    Command::new("up")
                        .about("Mark the current identity address as online"),
                    Command::new("down")
                        .about("Mark the current identity address as offline"),
                    Command::new("delete")
                        .about("Delete the current identity address, optionally with associated data")
                        .args([
                            Arg::new("force")
                                .help("Delete all data referred to by the deleted address")
                                .long("force")
                                .short('f')
                        ]),
                ]),
            //// \^-^/ Address management commands
            ////
            //// The contact book contains "virtual data", meaning
            //// associations of metadata for network addresses.  To
            //// make it easier to remember your friends on the
            //// network you can store personal notes on addresses you
            //// encounter.
            Command::new("contact")
                .about("Manage a private identity contact book, allowing for custom notes and tags")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("add")
                        .about("Add a new contact with optional filter data")
                        .args([
                            Arg::new("address")
                                .help("peer address to save as contact")
                                .action(ArgAction::Set),
                            Arg::new("note")
                                .short('n')
                                .help("Add a searchable note")
                                .action(ArgAction::Set),
                            Arg::new("tags")
                                .short('t')
                                .help("Add a set of searchable tags in the format '<key>=<val>'")
                                .action(ArgAction::Append),
                            Arg::new("trust")
                                .short('u')
                                .help("Set a custom trust level from 1 to 7")
                                .action(ArgAction::Set)
                        ]),
                    Command::new("delete")
                        .about("Delete existing contact entries via filters")
                        .args([
                            Arg::new("address")
                                .help("peer address to save as contact")
                                .action(ArgAction::Set),
                            Arg::new("note")
                                .short('n')
                                .help("Add a searchable note")
                                .action(ArgAction::Set),
                            Arg::new("tags")
                                .short('t')
                                .help("Add a set of searchable tags in the format '<key>=<val>'")
                                .action(ArgAction::Append),
                            Arg::new("trust")
                                .short('u')
                                .help("Set a custom trust level from 1 to 7")
                                .action(ArgAction::Set)
                        ])
                ]),
            
            //// =^-^= Subscription management
            ////
            //// A subscription listens to all incoming messages for a
            //// given network flood namespace.
            //
            // Ratcat can be used to
            //// fetch data from a subscription, or integrate with
            //// xargs to periodically run some custom command on the
            //// received message.
            ////
            //// Subscriptions can have additional data, for example
            //// igonring messages when the address is offline, or
            //// setting a pre-determined timeout, after which the
            //// subscription will be deleted, along with any data
            //// that hadn't been fetched before.
            Command::new("sub")
                .about("Manage subscriptions")
                .arg_required_else_help(true)
                .subcommands([
                    //// Add a new subscription, alongside additional data
                    Command::new("add")
                        .about("Add a new subscription for the current identity")
                        .args([
                            //// This has no bearing on other
                            //// subscriptions to a given namespace:
                            //// another address may have an async
                            //// subscription and thus packets may
                            //// still be collected
                            Arg::new("synced")
                                .help("Don't collect messages for the namespace when the address is offline")
                                .action(ArgAction::SetTrue),

                            //// Again, this MAY not have an effect on
                            //// the journal on disk, if another
                            //// subscription still has claim to some
                            //// of the same data.
                            Arg::new("timeout")
                                .help("Set a pre-determined destruct date for the subscription")
                                .action(ArgAction::Set)
                        ]),
                    //// Removing a subscription is very straight
                    //// forward, just one mandatory parameter is
                    //// required (which subscription to delete, for a
                    //// given address).
                    Command::new("rm")
                        .about("Remove an existing subscription for the current identity")
                        .arg(
                            Arg::new("namespace")
                                .help("Specify which subscription to delete")
                        ),
                ]),
            //// Query various types of status output
            Command::new("status")
                .about("See component status")
                .arg_required_else_help(true)
                .subcommands([
                    //// Print information about how many threads
                    //// Ratman is running, how much memory is being
                    //// used, how many clients and links are
                    //// connected, how many peers are seen online, what the network global latency is
                    Command::new("system").about("Print the overall system status"),
                    //// Print individual address statistics, for
                    //// example how much traffic that address has
                    //// produced in various timescales, etc
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
            Command::new("peer").about("List and query peers")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("query")
                        .about("Search for a specific peer via a query")
                        .args([
                            Arg::new("by note")
                                .short('n')
                                .action(ArgAction::Set)
                                .help("String search of contact notes"),
                            Arg::new("by tag")
                                .short('t')
                                .action(ArgAction::Set)
                                .help("Filter by a matching tag in the format '<key>=<val>'"),
                            Arg::new("trust above")
                                .long("tr-above")
                                .action(ArgAction::Set)
                                .help("Filter by a trust level higher (or equal) to the provided"),
                            Arg::new("trust below")
                                .long("tr-below")
                                .action(ArgAction::Set)
                                .help("Filter by a trust level below the provided")
                        ]),
                    Command::new("list")
                        .about("List and browse all available peers")
                        .arg(
                            Arg::new("interactive")
                                .short('i')
                                .help("Capture the terminal and provide an interactive address browser")
                                .action(ArgAction::SetTrue)
                        ),
                ]),
            Command::new("link")
                .about("Manage network links (netmods)")
                .before_help("Adding new netmods during runtime is not yet implemented.")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("list")
                        .about("List currently available netmod links"),
                    Command::new("up")
                        .about("Mark the given link as online"),
                    Command::new("down")
                        .about("Mark the given link as offline"),
                ]),
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

#[async_std::main]
async fn main() {
    let cli = setup_cli();
    let m = cli.get_matches();

    // if let Err(e) = ratman_tools::command_filter(m).await {
    //     eprintln!("Operation failed, an Error occurred:\n{}", e);
    // }
}
