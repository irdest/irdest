# Irdest Echo

This is a very simple application built on top of Ratman.  In fact, it is so simple we can include it in its entirety in this manual:

```rust
use ratman_client::{Identity, RatmanIpc, Receive_Type};

#[async_std::main]
async fn main() {
    let ipc = RatmanIpc::default()
        .await
        .expect("Failed to connect to Ratman daemon!");

    println!("Listening on address: {}", ipc.address());
    while let Some((tt, mut msg)) = ipc.next().await {
        // Ignore flood messages
        if tt == Receive_Type::FLOOD {
            continue;
        }

        // Get the message sender identity and reply
        let sender = Identity::from_bytes(msg.get_sender());
        ipc.send_to(sender, msg.take_payload()).await.unwrap();
    }
}
```

Fundamentally `irdest-echo` connects to a local Ratman router daemon
and waits for incoming messages.  For each message it takes the
payload and sends a message back to the sender of the message.

This demonstrates that mesh networking doesn't have to be hard for
application developers, without having to make assumptions about the
available transport channels.

We don't currently build `irdest-echo` as part of our CI pipeline so
you will have to build it yourself.

```console
$ cargo run --bin irdest-echo --release
```

## Public instance

The Irdest project maintains an `irdest-echo` instance at the
following addresses:

To send a message to `irdest-echo` you can use `ratcat` and the following command:

 - `fill in address...`
