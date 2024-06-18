use crate::{base_args::BaseArgs, parse_field, reply_ok};
use clap::ArgMatches;
use colored::Colorize;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio,
    types::{error::UserError, Address, Ident32, LetterheadV1, Recipient},
    Result,
};
use std::sync::Arc;

fn generate_letterheads(
    from: Address,
    to_addrs: &String,
    stream_size: u64,
    recp_maker: impl Fn(Address) -> Recipient,
) -> Result<Vec<LetterheadV1>> {
    let mut spaces = to_addrs
        .split(",")
        .map(|x| {
            eprintln!("post-split: {x}");
            x
        })
        .map(|s| s.trim())
        .map(|x| {
            eprintln!("post-trim {x}");
            x
        })
        .map(|chunk| Ident32::try_from(chunk).map(|id| Address(id)))
        .map(|x| {
            eprintln!("post-addressed {x:?}");
            x
        })
        .collect::<Result<Vec<_>>>()?;

    eprintln!("Collected recipients: {spaces:?}");

    let mut lh_buf = vec![];
    loop {
        if let Some(to) = spaces.pop() {
            eprintln!("Create letterhead for {to:?}");
            lh_buf.push(LetterheadV1 {
                from,
                to: recp_maker(to),
                stream_size,
                auxiliary_data: vec![],
            });
            continue;
        }

        break Ok(lh_buf);
    }
}

pub async fn send(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let to_addr = parse_field::<String>(matches, "to-address");
    let to_space = parse_field::<String>(matches, "to-space");
    let stream_size = *parse_field::<u64>(matches, "stream-size")?;

    eprintln!("Generate letterheads");
    let to = match (to_addr, to_space) {
        (Ok(to_addrs), Err(_)) => {
            generate_letterheads(addr, to_addrs, stream_size, |a| Recipient::Address(a))?
        }
        (Err(_), Ok(to_str)) => {
            generate_letterheads(addr, to_str, stream_size, |s| Recipient::Namespace(s))?
        }
        (Ok(to_addr_str), Ok(to_s_str)) => {
            let addrs =
                generate_letterheads(addr, to_addr_str, stream_size, |a| Recipient::Address(a))?;
            let spaces =
                generate_letterheads(addr, to_s_str, stream_size, |s| Recipient::Address(s))?;
            addrs.into_iter().chain(spaces.into_iter()).collect()
        }
        (Err(e1), Err(e2)) => {
            return Err(UserError::InvalidInput(
                format!("Failed to use addrs: {e1}\nFailed to use spaces:{e2}"),
                Some("A comma-separated list of either addresses or namespaces".to_string()),
            )
            .into());
        }
    };
    let mut stdin = tokio::io::stdin();

    //if !base_args.quiet {
    eprintln!(
        "Generated {} letterheads to {:?}",
        to.len(),
        to.iter()
            .map(|to| to.to.inner_address().pretty_string())
            .collect::<Vec<_>>()
    );

    ipc.send_many(auth, to, &mut stdin).await?;
    println!("{}", reply_ok(&base_args.out_fmt).as_str().green());
    Ok(())
}
