//! Ratman API library

use clap::ArgMatches;
use libratman::Result;

mod sub;

pub const RATS: &'static str = include_str!("../rats.ascii");

pub async fn command_filter(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some((cmd, operand)) => match operand.subcommand() {
            Some((op, op_matches)) => match (cmd, op) {
                //// =^-^= Address commands
                ("addr", "create") => Ok(()),
                ("addr", "destroy") => Ok(()),
                ("addr", "up") => Ok(()),
                ("addr", "down") => Ok(()),
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
