use crate::{base_args::BaseArgs, parse_field, reply_ok};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
    },
    types::{error::UserError, Address, Ident32, LetterheadV1, Recipient},
    Result,
};
use std::sync::Arc;

pub async fn receive(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let _count = matches.get_one::<u64>("stream-count").unwrap(); // has default
    let addr_to = matches.get_one::<String>("to-address").unwrap(); // required

    let mut stdout = tokio::io::stdout();
    let (letterhead, mut read_stream) = ipc
        .recv_one(
            auth,
            addr,
            Recipient::Address(Address(Ident32::try_from(addr_to.as_str())?)),
        )
        .await?;

    eprintln!(
        "Receiving message stream: {}",
        serde_json::to_string_pretty(&letterhead)?
    );

    tokio::io::copy(
        // Limit the amount of data this socket reads
        &mut read_stream.as_reader().take(letterhead.stream_size),
        &mut stdout,
    )
    .await?;

    read_stream.drop().await?;
    Ok(())
}
