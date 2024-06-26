use crate::{base_args::BaseArgs, encode_list};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    Result,
};
use std::sync::Arc;

pub async fn list(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, _matches: &ArgMatches) -> Result<()> {
    let peers_list = ipc.peers_list().await?;
    println!("{}", encode_list(peers_list, base_args.out_fmt));
    Ok(())
}
