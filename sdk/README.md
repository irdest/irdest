# Irdest SDKs

In order to develop applications for a irdest network, you need to
include the software development kit libraries (SDK libs).  Each
component your application wants to interact with requires a separate
library.  For the official core components all SDKs can be found in
this directory.  For third-party components, you may have to source
them from different development repositories.


## SDK structure

Each SDK exposes a set of types used by the component, with an
async/await API which uses a common RPC interface (called `qrpc`) to
facilitate communication and reply data.

Currently the following irdest components have SDK libraries:

- [libirdest](./libirdest-sdk)
- [irdest-chat](./irdestchat-sdk)


## Writing a irdest service

You can find much more detailed instructions on how to develop a irdest
service in the [developer manual]().  Following is a very simple
"hello world".

A small, but real-world example of a service can be found
[here](../services/ping).

```rust
// Requirue the RPC protocol core
use qrpc_sdk::{default_socket_path, RpcSocket, Service};

// Include the libqau component SDK
use libirdest_sdk::{IrdestSdk, types::UserAuth};

#[async_std::main]
async fn main() {
    let mut serv = Service::new("net.irdest.test", 1, "A test service");
    
    // Get the default qrpc socket location
    let (addr, port) = default_socket_path();
    
    // Connect your service to the qrpc backend
    let id = serv
        .register(RpcSocket::connect(addr, port).await.unwrap())
        .await
        .unwrap();
        
    println!("My service ID is '{}'", id);
}
```
