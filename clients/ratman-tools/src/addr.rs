use crate::{base_args::BaseArgs, encode_list, encode_map, reply_ok};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    Result,
};
use std::sync::Arc;

pub async fn list(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, _matches: &ArgMatches) -> Result<()> {
    let addrs_list = ipc.addr_list().await?;
    println!("{}", encode_list(addrs_list, base_args.out_fmt));
    Ok(())
}

pub async fn create(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, matches: &ArgMatches) -> Result<()> {
    let name = matches.get_one::<String>("priv-name");

    let (addr, auth) = ipc.addr_create(name).await?;

    println!(
        "{}",
        encode_map(
            // make sure we print the correct results (AddrAuth doesn't like
            // being printed so we go around it to get the inner token)
            vec![("addr", addr.to_string()), ("auth", auth.token.to_string())],
            base_args.out_fmt
        )
    );
    Ok(())
}

pub async fn up(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, _matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    ipc.addr_up(auth, addr).await?;
    println!("{}", reply_ok(&base_args.out_fmt));
    Ok(())
}

pub async fn down(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, _matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    ipc.addr_down(auth, addr).await?;
    println!("{}", reply_ok(&base_args.out_fmt));
    Ok(())
}

pub async fn destroy(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    _matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    ipc.addr_destroy(auth, addr, false).await?;
    println!("{}", reply_ok(&base_args.out_fmt));
    Ok(())
}
