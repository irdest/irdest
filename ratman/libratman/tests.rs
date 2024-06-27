use libratman::{RatmanIpc, Result, Recipient, tokio};

#[tokio::main]
async fn main() -> Result<()> {
    let ipc = RatmanIpc::start().await?;

    // Create a regular address
    // The second parameter can be used to create a specific namespace key
    let (addr, auth) = ipc.addr_create("my-client", None).await?;

    // Start advertising the address on the network
    ipc.addr_up(addr, auth).await?;

    // Wait for incoming messages
    let (letterhead, reader) = ipc.recv_one(addr, Recipient::Address(addr)).await?;

    println!("Receiving stream {letterhead:?}");
    let mut stdout = tokio::io::stdout();
    
    tokio::io::copy(&mut reader.as_reader(), &mut stdout).await?;
    Ok(())
}
