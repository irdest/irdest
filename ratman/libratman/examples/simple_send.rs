use libratman::{
    api::{default_api_bind, RatmanIpc, RatmanIpcExtV1, RatmanStreamExtV1},
    tokio,
    types::{LetterheadV1, Recipient},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let ipc = RatmanIpc::start(default_api_bind()).await?;

    // Create a regular address
    // The second parameter can be used to create a specific namespace key
    let (addr, auth) = ipc.addr_create(Some(&"my-name".to_owned()), None).await?;

    let peers = ipc.peers_list().await?;
    let friend = peers
        .first()
        .expect("You need to have a friend to talk to :(");

    // Start advertising the address on the network
    ipc.addr_up(auth, addr).await?;

    let mut stdin = tokio::io::stdin();

    ipc.send_to(
        auth,
        LetterheadV1::send(addr, Recipient::Address(friend.addr)),
        &mut stdin,
    )
    .await?;

    Ok(())
}
