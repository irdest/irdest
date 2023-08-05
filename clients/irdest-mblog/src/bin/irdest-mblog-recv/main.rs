// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use anyhow::Result;
use clap::Parser;
use irdest_mblog::{Message, Payload};
use libratman::{client::RatmanIpc, types::Address};
use std::convert::TryFrom;

/// sample microblog client - cli receiver.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Address to use. Automatic if not set.
    #[clap(long)]
    addr: Option<String>,
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Take an address from --addr, load it from disk, or generate and save one.
    // if let Some(addr_str) = args.addr {
    //     Address::from_string(&addr_str)
    // } else {
    let (_, addr, token) = irdest_mblog::load_or_create_addr().await?;
    // };

    let ipc = RatmanIpc::default_with_addr(addr, token).await?;
    while let Some((tt, ratmsg)) = ipc.next().await {
        match Message::try_from(&ratmsg) {
            Ok(msg) => match msg.payload {
                Payload::Post(p) => println!("{}: {}", p.nick, p.text),
            },
            Err(e) => {
                eprintln!("[invalid message]: {:}\n{:?} {:?}", e, tt, ratmsg);
            }
        };
    }

    Ok(())
}
