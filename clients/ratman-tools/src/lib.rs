//! Ratman API library

use base_args::BaseArgs;
use clap::{Arg, ArgAction, ArgMatches};
use libratman::{
    api::RatmanIpc,
    tokio::runtime::{Builder, Runtime},
    types::{error::UserError, Ident32},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::BTreeMap, fmt::Display, sync::Arc};

pub mod addr;
pub mod base_args;
pub mod peers;
pub mod recv;
pub mod send;
pub mod space;
pub mod status;
pub mod stream;

pub const RATS: &'static str = include_str!("../rats.ascii");

pub fn global_args() -> Vec<Arg> {
    vec![
        Arg::new("api-bind")
            .action(ArgAction::Set)
            .help("Override the default client API socket address")
            .short('b')
            .long("bind")
            .default_value("127.0.0.1:5852"),
        Arg::new("state-dir")
            .action(ArgAction::Set)
            .help("Override the state/config directory")
            .short('d')
            .long("dir"),
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
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Lines,
    Json,
}

pub fn tokio_runtime() -> Runtime {
    match Builder::new_current_thread().enable_all().build() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to start async runtime :( {e}");
            eprintln!("this is a very uncommon error, please report it to us!");
            std::process::exit(2);
        }
    }
}

pub(crate) fn parse_ident32(m: &ArgMatches, key: &str) -> Result<Ident32> {
    match m.get_one::<String>(key) {
        Some(k) => match Ident32::try_from_bytes(k.as_bytes()) {
            Ok(id) => Ok(id),
            // In-case the parsing failed we try to strip quotes from the input
            // and then try again.  If this still fails then we just bail
            _ => Ident32::try_from_bytes(&k.replace('"', "").to_string().as_bytes()),
        },
        None => Err(UserError::MissingInput(format!("Input {key} was not provided")).into()),
    }
}

pub(crate) fn parse_field<'m, T: Any + Sync + Send + Clone>(
    m: &'m ArgMatches,
    key: &str,
) -> Result<&'m T> {
    Ok(m.get_one::<T>(key).ok_or(UserError::MissingInput(
        "Required input {key} is missing!".to_owned(),
    ))?)
}

pub(crate) fn encode(tt: impl Display + Serialize, fmt: OutputFormat) -> String {
    match fmt {
        OutputFormat::Json => serde_json::to_string_pretty(&tt).unwrap(),
        OutputFormat::Lines => format!("{}", tt),
    }
}

pub(crate) fn encode_map<
    I: IntoIterator<Item = (K, V)>,
    K: Display + Serialize,
    V: Display + Serialize,
>(
    iter: I,
    fmt: OutputFormat,
) -> String {
    match fmt {
        OutputFormat::Json => serde_json::to_string_pretty(
            &iter
                .into_iter()
                .map(|(k, v)| (format!("{k}"), format!("{v}")))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap(),
        OutputFormat::Lines => iter
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

pub(crate) fn encode_list<I: IntoIterator<Item = V>, V: Display + Serialize>(
    iter: I,
    fmt: OutputFormat,
) -> String {
    match fmt {
        OutputFormat::Json => serde_json::to_string(&iter.into_iter().collect::<Vec<V>>()).unwrap(),
        OutputFormat::Lines => iter
            .into_iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join("\n"),
    }
}

pub(crate) fn reply_ok(output_format: &OutputFormat) -> String {
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

/// This function handles both ratcat and ratctl (why are they not the same CLI
/// anyway??).
///
/// To make it less boilerplate-y we handle all commands here.  The comment
/// above each section indicates whether it's run by rat(cat) or rat(ctl).
pub async fn command_filter(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: ArgMatches,
) -> Result<()> {
    match matches.subcommand() {
        Some((cmd, operand)) => {
            match operand.subcommand() {
                Some((op, op_matches)) => match (cmd, op) {
                    //// =^-^= Address commands (ctl)
                    ("addr", "create") => addr::create(ipc, base_args, op_matches).await,
                    ("addr", "destroy") => addr::destroy(ipc, base_args, op_matches).await,
                    ("addr", "up") => addr::up(ipc, base_args, op_matches).await,
                    ("addr", "down") => addr::down(ipc, base_args, op_matches).await,
                    ("addr", "list") => addr::list(ipc, base_args, op_matches).await,
                    //// =^-^= Status commands (ctl)
                    ("status", "system") => status::system(ipc, base_args, op_matches).await,
                    //// =^-^= Peer commands (ctl)
                    ("peers", "list") => peers::list(ipc, base_args, op_matches).await,
                    //// =^-^= Namespace commands (ctl)
                    ("space", "generate") => space::generate(ipc, base_args, op_matches).await,
                    ("space", "load") => space::load(ipc, base_args, op_matches).await,
                    ("space", "up") => space::up(ipc, base_args, op_matches).await,
                    ("space", "down") => space::down(ipc, base_args, op_matches).await,
                    ("space", "anycast") => space::anycast(ipc, base_args, op_matches).await,
                    //// =^-^= Stream subscription commands (ctl)
                    ("stream", "sub") => stream::subscribe(ipc, base_args, op_matches).await,
                    ("stream", "unsub") => stream::unsubscribe(ipc, base_args, op_matches).await,
                    ("stream", "resub") => stream::resubscribe(ipc, base_args, op_matches).await,
                    //// =^-^= House-keeping and meta commands
                    ("idpath", _) => {
                        println!("{}", base_args.identity_path);
                        Ok(())
                    }
                    _ => unreachable!("oops! looks like the cli library didn't filter this"),
                },
                None => match cmd {
                    //// =^-^= Send commands (cat)
                    "send" => send::send(ipc, base_args, operand).await,
                    "recv" => recv::receive(ipc, base_args, &operand).await,
                    //// =^-^= House-keeping and meta commands
                    "idpath" => {
                        println!("{}", base_args.identity_path);
                        // Completely bail out here to avoid printing the "statistics page" for ratcat
                        // which will be very silly on a command that only prints a single line
                        std::process::exit(0);
                    }
                    _ => unreachable!("oops! looks like the cli library didn't filter this"),
                },
            }
        }
        _ => unreachable!("oops! looks like the cli library didn't filter this"),
    }
}
