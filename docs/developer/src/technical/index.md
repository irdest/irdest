# Technical Documentation

Welcome to the irdest technical documentation.  Following is an
outline of the system, which will hopefully give you an initial
understanding of the project stack.

This manual is relevant for both **irdest hackers**, and **irdest
application developers**.

**Note to future editors**: make sure every major component has an
`internals` section, so to make it convenient to skip.


## Introduction

Fundamentally, irdest is a highly distributed system.  It is not
accountable to a single set of rules across all devices that are part
of this system.  Each device can be home to many users, that can all
send messages to each other, but have separate states.  Furthermore,
connections in this system are not always real time.

A lot of traditional networking infrastructe is built up in layers.
Similarly, the irdest project replicates these layers.  To give you a
better understanding of what parts of the irdest project do what,
here's an overview of the layers in a full irdest application stack.


| Layer            | Component name                      | Description                                                                                                   |
|------------------|-------------------------------------|---------------------------------------------------------------------------------------------------------------|
| Network drivers  | `netmod`, `netmod-tcp`, ...         | Network drivers to establish and manage connections with peers via different underlying transport mechansms   |
| Irdest Router    | `ratman`                            | A decentralised packet router using cryptographic public keys as addresses for users                          |
| Service API core | `irdest-core`                       | The main irdest state handler, managing an encrypted database, user profiles, and more                        |
| Irdest RPC       | `irpc-sdk`, `irp-broker`            | A generic RPC protocol to connect different irdest services together                                          |
| Services         | `irdest-chat`, `irdest-groups`, ... | A service is a micro application (or [actor]) that communicates with other services via the irpc bus          |
| Clients          | `irdest-hubd`, `irdest-gtk`         | User facing applications that uses a collection of services and the RPC layer to implement different features |
    
[actor]: https://en.wikipedia.org/wiki/Actor_(programming_language)


### Services & clients

A service is a client on the irdest RPC interface.  A service MAY
expose an API to other services.  An example of such a service is
`irdest-groups`, which implements encrypted group management logic.

A service that does not expose an API of its own can be referred to as
a "client service".  Technically all user-interfaces are client
services.

The irdest project develops and ships a set of core services, that can
all be used via the `irdest-sdk` library.  Check out the [Rust
documentation][irdest-sdk] to find out how you can write a simple
client service using these existing APIs.

[irdest-sdk]: https://docs.irde.st/api/irdest_sdk/index.html


### Irdest RPC

The irdest RPC interface uses a central broker (called irpc-broker)
which listens for connections over local TCP connections.  This broker
is included in the `irdest-hubd` client.  A development utility client
is available: [`irpc-client`]().

*TODO: link to example service section*


### Irdest core

The irdest-core manages user identities.

### Ratman

This is arguably the heart of the irdest application stack.  Ratman is
a fully decentralised, delay tolerant gossip router.  It provides an
API that takes messages to send to peers on the network, and returns
messages received from the network.

It handles announcements, network segmentation, message journaling,
route updates, and networked archive storage.

Addresses on a Ratman network are 32-byte ed25519 public keys, meaning
that all direct messages are automatically encrypted.

### Network Modules

...
