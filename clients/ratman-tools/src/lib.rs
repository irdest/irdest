//! Ratman API library

use base_args::BaseArgs;
use clap::ArgMatches;
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
pub mod send;
pub mod stream;

pub const RATS: &'static str = include_str!("../rats.ascii");

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
        Some((cmd, operand)) => match operand.subcommand() {
            Some((op, op_matches)) => match (cmd, op) {
                //// =^-^= Address commands (ctl)
                ("addr", "create") => addr::create(ipc, base_args, op_matches).await,
                ("addr", "destroy") => addr::destroy(ipc, base_args, op_matches).await,
                ("addr", "up") => addr::up(ipc, base_args, op_matches).await,
                ("addr", "down") => addr::down(ipc, base_args, op_matches).await,
                ("addr", "list") => addr::list(ipc, base_args, op_matches).await,
                //// =^-^= Receive commands (cat)
                ("recv", "one") => Ok(()),
                ("recv", "many") => Ok(()),

                //// =^-^= Status commands (ctl)
                ("status", "system") => Ok(()),
                ("status", "addr") => Ok(()),
                ("status", "sub") => Ok(()),
                //// =^-^= Stream subscription commands (ctl)
                ("stream", "sub") => stream::subscribe(ipc, base_args, op_matches).await,
                ("stream", "unsub") => stream::unsubscribe(ipc, base_args, op_matches).await,
                ("stream", "resub") => stream::resubscribe(ipc, base_args, op_matches).await,
                _ => unreachable!("oops! looks like the cli library didn't filter this"),
            },
            None => match cmd {
                //// =^-^= Send commands (cat)
                "send" => send::send(ipc, base_args, operand).await,
                _ => unreachable!("oops! looks like the cli library didn't filter this"),
            },
        },
        _ => unreachable!("oops! looks like the cli library didn't filter this"),
    }
}
