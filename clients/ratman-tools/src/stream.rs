use crate::{encode_list, encode_map, parse_ident32, reply_ok, base_args::BaseArgs};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    types::{error::UserError, Address, Recipient},
    Result,
};
use std::sync::Arc;

pub async fn subscribe(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let sub_addr = parse_ident32(&matches, "address").ok();
    let sub_space = parse_ident32(&matches, "namespace").ok();

    let subscribe_to = match (sub_addr, sub_space) {
        (Some(sub_addr), None) => Recipient::Address(Address(sub_addr)),
        (None, Some(space_addr)) => Recipient::Namespace(Address(space_addr)),
        _ => {
            return Err(UserError::InvalidInput(
                "Must provide either --addr or --space".into(),
                None,
            )
            .into());
        }
    };

    // Since this program is about to shut down, we must print enough
    // information so the user can spawn their own subscriber.
    let mut subs_handle = ipc.subs_create(auth, addr, subscribe_to).await?;
    let sub_socket = subs_handle.peer_info();
    let sub_id = subs_handle.sub_id();

    println!(
        "{}",
        encode_map(
            vec![("sub_id", sub_id.to_string()), ("socket", sub_socket)],
            base_args.out_fmt
        )
    );

    Ok(())
}

pub async fn unsubscribe(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let sub_id = parse_ident32(&matches, "sub_id")?;
    ipc.subs_delete(auth, addr, sub_id).await?;
    println!("{}", reply_ok(&base_args.out_fmt));
    Ok(())
}

pub async fn resubscribe(
    ipc: &Arc<RatmanIpc>,
    base_args: BaseArgs,
    matches: &ArgMatches,
) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let sub_id = parse_ident32(&matches, "sub_id")?;

    // Since this program is about to shut down, we must print enough
    // information so the user can spawn their own subscriber.
    let mut subs_handle = ipc.subs_restore(auth, addr, sub_id).await?;
    let sub_socket = subs_handle.peer_info();
    let sub_id = subs_handle.sub_id();

    println!(
        "{}",
        encode_map(
            vec![("sub_id", sub_id.to_string()), ("socket", sub_socket)],
            base_args.out_fmt
        )
    );

    Ok(())
}

pub async fn list(ipc: &Arc<RatmanIpc>, base_args: BaseArgs, _matches: &ArgMatches) -> Result<()> {
    let (addr, auth) = base_args.identity_data?;
    let available_subs = ipc.subs_available(auth, addr).await?;
    println!("{}", encode_list(available_subs, base_args.out_fmt));
    Ok(())
}
