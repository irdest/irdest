//! Ratman API library

use clap::ArgMatches;
use libratman::{types::Ident32, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr};

pub mod addr;

pub const RATS: &'static str = include_str!("../rats.ascii");

pub struct BaseArgs {
    pub api_bind: Option<SocketAddr>,
    pub curr_id: Option<String>,
    pub profile: Option<String>,
    pub out_fmt: Option<OutputFormat>,
    pub quiet: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum OutputFormat {
    Lines,
    Json,
}

pub(crate) fn parse_ident32(m: &ArgMatches, key: &str) -> Option<Ident32> {
    match m.get_one::<String>("key") {
        Some(k) => match Ident32::try_from_bytes(k.as_bytes()).ok() {
            Some(id) => Some(id),
            None => Some(Ident32::from_string(&k.replace('"', "").to_string())),
        },
        None => None,
    }
}

pub(crate) fn encode_output<
    V: IntoIterator<Item = (K, V)>,
    K: Serialize + ToString,
    V: Serialize + ToString,
>(
    iter: V,
    fmt: OutputFormat,
) -> String {
    match fmt {
        OutputFormat::Json => {
            serde_json::to_string(&iter.into_iter().collect::<BTreeMap<K, V>>()).unwrap()
        }
        OutputFormat::Lines => iter
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
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

pub async fn command_filter(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some((cmd, operand)) => match operand.subcommand() {
            Some((op, op_matches)) => match (cmd, op) {
                //// =^-^= Address commands
                ("addr", "create") => Ok(()),
                ("addr", "destroy") => Ok(()),
                ("addr", "up") => Ok(()),
                ("addr", "down") => Ok(()),
                ("addr", "list") => Ok(()),
                //// =^-^= Contact commands
                ("contact", "add") => Ok(()),
                ("contact", "delete") => Ok(()),
                ("contact", "modify") => Ok(()),
                //// =^-^= Link commands
                ("link", "up") => Ok(()),
                ("link", "down") => Ok(()),
                ("link", "list") => Ok(()),
                //// =^-^= Peer commands
                ("peer", "query") => Ok(()),
                ("peer", "list") => Ok(()),
                //// =^-^= Receive commands
                ("recv", "one") => Ok(()),
                ("recv", "many") => Ok(()),
                ("recv", "fetch") => Ok(()),
                //// =^-^= Send commands
                ("send", "one") => Ok(()),
                ("send", "many") => Ok(()),
                ("send", "flood") => Ok(()),
                //// =^-^= Status commands
                ("status", "system") => Ok(()),
                ("status", "addr") => Ok(()),
                ("status", "link") => Ok(()),
                //// =^-^= Subscription commands
                ("sub", "add") => sub::add(op_matches).await,
                ("sub", "rm") => sub::rm(op_matches).await,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
