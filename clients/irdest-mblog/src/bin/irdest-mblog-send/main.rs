// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use anyhow::Result;
use clap::Parser;
use irdest_mblog::{Message, Post, NAMESPACE};
use libratman::{client::RatmanIpc, types::Address};
use protobuf::Message as _;

/// sample microblog client - cli sender.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Address to use. Automatic if not set.
    #[clap(long)]
    addr: Option<String>,

    /// Specify a nickname.
    #[clap(short, long, required = true)]
    nick: String,

    /// Topic to post to, eg. `general`.
    #[clap(short, long, required = true)]
    topic: String,

    /// Text to send.
    #[clap()]
    text: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Take an address from --addr, load it from disk, or generate and save one.
    let (_, addr, token) =
    //     if let Some(addr_str) = args.addr {
    //     Address::from_string(&addr_str)
    // } else {
        irdest_mblog::load_or_create_addr().await?;
    // };

    // Create a message.
    let msg = Message::new(Post {
        nick: args.nick,
        text: args.text,
        topic: args.topic,
    });

    // Connect and send!
    RatmanIpc::default_with_addr(addr, token)
        .await?
        .flood(NAMESPACE.into(), msg.into_proto().write_to_bytes()?, true)
        .await?;

    Ok(())
}
