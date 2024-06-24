use crate::base_args::BaseArgs;
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio::{self, io::AsyncReadExt},
    types::{Address, Ident32, Recipient},
    Result,
};
use std::sync::Arc;

pub async fn receive(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let count = matches.get_one::<u64>("stream-count").unwrap_or(&1);

    let addr_to = Address(
        matches
            .get_one::<String>("to-address")
            .and_then(|buf| Ident32::try_from(buf.as_str()).ok())
            .unwrap_or_else(|| addr.0),
    );

    let mut stdout = tokio::io::stdout();
    let mut step = *count;
    loop {
        let (letterhead, mut read_stream) =
            match ipc.recv_one(auth, addr, Recipient::Address(addr_to)).await {
                Ok((lh, rs)) => (lh, rs),
                Err(e) => {
                    eprintln!("Read stream ended: {e}");
                    break;
                }
            };

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

        if step == 1 {
            break;
        }

        step -= 1;
    }

    Ok(())
}
