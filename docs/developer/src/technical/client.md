# Ratman client lib

This is the client library used to write applications for Ratman.  Currently only a [Rust](https://rust-lang.org) implementation exists.  The raw protocol is documented in the next chapter.

You can find `ratman-client` on
[crates.io](https://crates.io/crates/ratman-client) and its documentation on
[docs.rs](https://docs.rs/ratman-client)!


## Workflow

There are three main steps to using the client-lib:

1. IPC initialisation
2. Address registration/ login
3. Message sending and receiving


### IPC initialisation

By default the IPC socket for Ratman is running on `localhost:5852`. Many of the Irdest tools allow you to overwrite this socket address, to allow for local testing with multiple routers.  We recommend that your application expose this option to users for testing purposes as well!


### Address registration

An address for Ratman is associated with a cryptographic key pair.  Currently we don't expose the private key from the router to applications (which will probably change in the future!)

When your application is given an address you should store it in your application state somewhere, along with the corresponding address auth token.  These will be important the next time your application starts.  *For privacy reasons you should encrypt this data with a user password!*


### Message sending and receiving

Every "message" in Irdest is a stream, which is encoded into encrypted data blocks, along with a manifest which contains the root block reference and key.  Currently the manifest is not encrypted, meaning that anyone intercepting it will be able to decode the rest of the block stream.

Sending a message stream happens somewhat asynchronously:  the sending client will block until the router has received the full message, but there's currently no feedback how encoding or sending is going.

Receiving message streams can happen in two ways: synchronously and asynchronously via subscriptions.  A subscription can persist across router restarts and will save missed items for a client to re-play when it next connects.  Synchronous receiving blocks the main client socket, so no other commands can be exchanged.  A subscription uses a dedicated TCP socket for a client to connect to.

When receiving you first get a "letterhead", which contains who the stream is from, who it is addressed to, it's final size, and additional metadata map currently not used).

Messages can either be sent to one ore multiple *Addresses* or a *Namespace*.  Address-recipient messages will only be delivered to the devices that have advertised those addresses.

Namespaces are special addresses that any client can subscribe to.  Namespace-recipient messages are spread across the whole network and any router/ client that subscribes to it can receive them.  Namespaces are a great way for different instances of your application to find each other.

So for example a message sent to the address `ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6` will be carried across the network until it reaches _this exact_ address.

A message sent to the namespace
`ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6` will be delivered to all applications that are _listening_ on this namespace.


### API example

This is a small program demonstrating the most basic usage of the ratman-client SDK.  At start-up it registers a new address, listens to any incoming messages, and returns them as they are to the sender.

```rust
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
```
