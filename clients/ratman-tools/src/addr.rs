use crate::{encode_output, parse_ident32, reply_ok, BaseArgs};
use clap::ArgMatches;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    types::{error::UserError, Ident32},
    RatmanError,
};

pub async fn list(ipc: &mut RatmanIpc, base_args: BaseArgs, _matches: ArgMatches) -> Result<()> {
    let addrs_list = ipc.addr_list().await?;


    println!("{}", encode_output(iter, fmt));
    
    match output_format {
        OutputFormat::Json => {
            return Ok(serde_json::to_string(&addrs_list)?);
        }
        OutputFormat::Lines => {
            return Ok(format!(
                "{}",
                addrs_list
                    .into_iter()
                    .map(|x| x.pretty_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
}

pub async fn create(ipc: &mut RatmanIpc, base_args: BaseArgs, matches: ArgMatches) -> Result<()> {
    let space_data = parse_ident32(&matches, "namespace").ok_or(RatmanError::User(
        UserError::InvalidInput("Provided namespace address is not valid".into(), None),
    ))?;
    let name = matches.get_one::<String>("priv-name");

    let (addr, auth) = ipc.addr_create(name, space_data).await?;

    println!(
        "{}",
        encode_output(vec![("addr", addr), ("auth", auth)], base_args.out_fmt)
    );
    Ok(())
}

pub async fn up(ipc: &mut RatmanIpc, base_args: BaseArgs, matches: ArgMatches) -> Result<()> {
    let (addr, auth) = identity_data.unwrap();
    ipc.addr_up(auth, addr).await?;
    println!("{}", reply_ok(&output_format));
    Ok(())
}

pub async fn down(ipc: &mut RatmanIpc, base_args: BaseArgs, matches: ArgMatches) -> Result<()> {
    let (addr, auth) = identity_data.unwrap();
    ipc.addr_down(auth, addr).await?;
    println!("{}", reply_ok(&output_format));
    Ok(())
}

pub async fn destroy(ipc: &mut RatmanIpc, base_args: BaseArgs, matches: ArgMatches) -> Result<()> {
    let (addr, auth) = identity_data.unwrap();
    ipc.addr_destroy(auth, addr, true).await?;
    println!("{}", reply_ok(&output_format));
    Ok(())
}
