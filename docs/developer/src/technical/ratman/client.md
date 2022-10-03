# Ratman client lib

This is the client library used to write applications for Ratman.
Currently only a [Rust](https://rust-lang.org) implementation exists.

You can find `ratman-client` on
[crates.io](https://crates.io/crates/ratman-client) and its documentation on
[docs.rs](https://docs.rs/ratman-client)!


## Workflow

There are three main steps to using the client-lib:

1. IPC initialisation
2. Address registration/ login
3. Message sending and receiving


### IPC initialisation

By default the IPC socket for Ratman is running on `localhost:9020`.
Many of the Irdest tools allow you to overwrite this socket address,
to allow for local testing with multiple routers.  We recommend that
your application expose this option to users for testing purposes as
well!


### Address registration

An address for Ratman is associated with a cryptographic key pair.
Currently we don't expose the private key from the router to
applications (which will probably change in the future!)

When your application is given an address you should store it in your
application state somewhere, along with the corresponding auth token.
These will be important the next time your application starts.  *For
added security you should encrypt this data with a user password!*


### Message sending and receiving

Sending messages happens asynchronously, which means that the client
lib will not get feedback on if your message has actually been
dispatched across a network channel, let alone been received.

Messages can be sent as one of two types: **Standard** and **Flood**.

**Standard** messages have a fixed recipient address and will be
routed to the destination where they will leave the network and be
processed by the target application (or dropped).

**Flood** messages are sent to every device and address on the
network, to allow for network-wide announcements (this is also how
your address announces itself!)  To limit the amount of relevant Flood
messages an application has to deal with, they are namespaced.  The
namespace itself is also an Irdest address.

So for example a **standard** message sent to
`ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6`
will be carried across the network until it reaches _this exact_
address.

A **flood** message sent to
`ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6`
will be delivered to all applications that are _listening_ on this
namespace.


### API Example: `irdest-echo`

This is a small program demonstrating the most basic usage of the
ratman-client SDK.  At start-up it registers a new address, listens to
any incoming messages, and returns them as they are to the sender.

```rust
use ratman_client::{RatmanIpc, Receive_Type};

#[async_std::main]
async fn main() {
    let ipc = RatmanIpc::default()
        .await
        .expect("Failed to connect to Ratman daemon!");

    println!("Listening on address: {}", ipc.address());
    while let Some((tt, msg)) = ipc.next().await {
        // Ignore flood messages
        if tt == Receive_Type::FLOOD {
            continue;
        }

        // Get the message sender identity and reply
        let sender = msg.get_sender();
        ipc.send_to(sender, msg.get_payload()).await.unwrap();
    }
}
```
