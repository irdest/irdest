use crate::{base_args::BaseArgs, parse_field, reply_ok};
use clap::ArgMatches;
use colored::Colorize;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio::{self, io::AsyncReadExt},
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
        .map(|s| s.trim())
        .map(|chunk| Ident32::try_from(chunk).map(|id| Address(id)))
        .collect::<Result<Vec<_>>>()?;

    let mut lh_buf = vec![];
    loop {
        if let Some(to) = spaces.pop() {
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

#[inline(never)]
pub async fn send(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let to_addr = parse_field::<String>(matches, "to-address");
    let to_space = parse_field::<String>(matches, "to-space");
    let chunk_size_str: String = parse_field::<String>(matches, "chunk-size")
        .map(|x| x.to_lowercase())
        .unwrap_or("0".to_string());

    let chunk_size: u64 = if chunk_size_str.contains("k") {
        chunk_size_str
            .replace("k", "")
            .parse::<u64>()
            .map_err(|_| {
                UserError::InvalidInput(
                    chunk_size_str.replace("k", "").to_string(),
                    Some(format!("valid integer")),
                )
            })?
            * /* Kibibytes */ 1024
    } else if chunk_size_str.contains("m") {
        chunk_size_str
            .replace("m", "")
            .parse::<u64>()
            .map_err(|_| {
                UserError::InvalidInput(
                    chunk_size_str.replace("m", "").to_string(),
                    Some(format!("valid integer")),
                )
            })?
            * /* Mibibytes */ (1024 * 1024)
    } else {
        chunk_size_str.parse().map_err(|_| {
            UserError::InvalidInput(chunk_size_str.to_string(), Some(format!("valid integer")))
        })?
    };

    eprintln!("Generate letterheads");
    let to = match (to_addr, to_space) {
        (Ok(to_addrs), Err(_)) => {
            generate_letterheads(addr, to_addrs, chunk_size, |a| Recipient::Address(a))?
        }
        (Err(_), Ok(to_str)) => {
            generate_letterheads(addr, to_str, chunk_size, |s| Recipient::Namespace(s))?
        }
        (Ok(to_addr_str), Ok(to_s_str)) => {
            let addrs = generate_letterheads(addr, to_addr_str, chunk_size.clone(), |a| {
                Recipient::Address(a)
            })?;
            let spaces =
                generate_letterheads(addr, to_s_str, chunk_size, |s| Recipient::Address(s))?;
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

    if chunk_size == 0 {
        eprintln!("Send full stream...");
        let mut stdin = tokio::io::stdin();
        ipc.send_many(auth, to, &mut stdin).await?;
    } else {
        loop {
            let mut stdin = tokio::io::stdin().take(chunk_size);
            eprintln!("Send {chunk_size} sized stream chunk...");
            ipc.send_many(auth, to.clone(), &mut stdin).await?;
        }
    }

    println!("{}", reply_ok(&base_args.out_fmt).as_str().green());
    Ok(())
}
