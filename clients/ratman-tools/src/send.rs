use crate::{base_args::BaseArgs, parse_field};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanStreamExtV1},
    tokio,
    types::{error::UserError, Address, Ident32, LetterheadV1, Recipient},
    Result,
};
use std::sync::Arc;

fn generate_letterheads(
    from: Address,
    to_addrs: &Vec<String>,
    stream_size: u64,
    recp_maker: impl Fn(Address) -> Recipient,
) -> Result<Vec<LetterheadV1>> {
    let mut spaces = to_addrs
        .into_iter()
        .map(|s| s.trim())
        .map(|s| s.as_bytes())
        .map(|chunk| Ident32::try_from(chunk).map(|id| Address(id)))
        .collect::<Result<Vec<_>>>()?;

    loop {
        let mut lh_buf = vec![];
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

// pub async fn one(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, matches: &ArgMatches) -> Result<()> {
//     let (addr, auth) = base_args.identity_data?;
//     let to_addr = parse_ident32(matches, "to-address");
//     let to_space = parse_ident32(matches, "to-space");
//     let stream_size = *parse_field::<u64>(matches, "stream-size")?;

//     let to = match (to_addr, to_space) {
//         (Ok(to_addr), Err(_)) => Recipient::Address(Address(to_addr)),
//         (Err(_), Ok(to_space)) => Recipient::Namespace(Address(to_space)),
//         _ => {
//             return Err(
//                 UserError::MissingInput("Must provide either -a or -s to send".into()).into(),
//             )
//         }
//     };

//     let lh = LetterheadV1 {
//         from: addr,
//         to,
//         stream_size,
//         auxiliary_data: vec![],
//     };

//     let mut stdin = libratman::tokio::io::stdin();

//     ipc.send_to(auth, lh, &mut stdin).await?;

//     Ok(())
// }

pub async fn send(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let to_addr = parse_field::<Vec<String>>(matches, "to-address");
    let to_space = parse_field::<Vec<String>>(matches, "to-space");
    let stream_size = *parse_field::<u64>(matches, "stream-size")?;

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

    ipc.send_many(auth, to, &mut stdin).await?;
    Ok(())
}
