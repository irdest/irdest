use crate::{base_args::BaseArgs, encode};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    Result,
};
use std::sync::Arc;

pub async fn system(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    _matches: &ArgMatches,
) -> Result<()> {
    let status = ipc.router_status().await?;
    println!("{}", encode(status, base_args.out_fmt));
    Ok(())
}
