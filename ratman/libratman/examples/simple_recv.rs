use libratman::{
    api::{default_api_bind, RatmanIpc, RatmanIpcExtV1, RatmanStreamExtV1},
    tokio,
    types::Recipient,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let ipc = RatmanIpc::start(default_api_bind()).await?;

    // Create a regular address
    // The second parameter can be used to create a specific namespace key
    let (addr, auth) = ipc.addr_create(Some(&"my-name".to_owned()), None).await?;

    // Start advertising the address on the network
    ipc.addr_up(auth, addr).await?;

    // Wait for incoming messages
    let (letterhead, mut reader) = ipc.recv_one(auth, addr, Recipient::Address(addr)).await?;

    println!("Receiving stream {letterhead:?}");
    let mut stdout = tokio::io::stdout();

    tokio::io::copy(&mut reader.as_reader(), &mut stdout).await?;
    Ok(())
}
