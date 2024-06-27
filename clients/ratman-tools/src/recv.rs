use crate::base_args::BaseArgs;
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
    },
    types::{error::UserError, Address, Ident32, Recipient},
    RatmanError, Result,
};
use std::sync::Arc;

#[inline(never)]
pub async fn receive(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let count = matches.get_one::<u64>("streams-count").unwrap_or(&1);

    let addr_to = Address(
        matches
            .get_one::<String>("to-address")
            .and_then(|buf| Ident32::try_from(buf.as_str()).ok())
            .unwrap_or_else(|| addr.0),
    );

    let mut stream_gen = ipc
        .recv_many(
            auth,
            addr,
            Recipient::Address(addr_to),
            // A count of 0 means we want to listen forever
            if *count == 0 {
                None
            } else {
                Some(*count as u32)
            },
        )
        .await?;

    loop {
        if !base_args.quiet {
            eprintln!("Waiting...");
        }

        let lh = match stream_gen.wait_for_manifest().await {
            Ok(letterhead) => letterhead,
            Err(RatmanError::User(UserError::RecvLimitReached)) => break,
            Err(e) => return Err(e),
        };

        if !base_args.quiet {
            eprintln!(
                "Receiving message stream: {}",
                serde_json::to_string_pretty(&lh)?
            );
        }

        let mut stdout = tokio::io::stdout();
        tokio::io::copy(
            // Limit the amount of data this socket reads
            &mut stream_gen.inner.as_reader().take(lh.stream_size),
            &mut stdout,
        )
        .await?;
        if !base_args.quiet {
            eprintln!("Next slide please...");
        }
        stdout.flush().await?;
    }

    Ok(())
}
