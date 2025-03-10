use std::time::Duration;

use libratman::{
    api::{default_api_bind, RatmanIpc, RatmanIpcExtV1, RatmanSpaceExt},
    generate_space_key, tokio, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let ipc = RatmanIpc::start(default_api_bind()).await?;

    // Create a regular address
    let (addr, auth) = ipc.addr_create(Some(&"my-name".to_owned())).await?;

    // Start advertising the address on the network
    ipc.addr_up(auth, addr).await?;

    // Create the space key and register it with the router
    let (space_addr, space_key) = generate_space_key();
    ipc.space_load(space_addr, space_key).await?;

    // Send an anycast probe to all other space participants
    let peers = ipc
        .space_anycast_probe(addr, auth, space_addr, Duration::from_millis(500))
        .await?;

    println!("Found these peers on the network {peers:?}");

    Ok(())
}
