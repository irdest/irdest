![](../docs/ratman-banner.png)

A modular userspace packet router, providing decentralised distance
vector routing and message delay tolerance.  Ratman is both a Rust
library and stand-alone and zero-config routing daemon.

Ratman provides a 256-bit address space via ed25519 public keys.  This
means that messages are end-to-end encrypted and authenticated at the
transport level (not currently implemented!).  An address is announced
via gossip protocol routing to other network participants.

Fundamentally Ratman exists outside the OSI layer scopes.  On one side
it binds to different transport layers (for example IP, or BLE) via
"net module" drivers.  On the other side it provides a simple protobuf
API on a local TCP socket for applications to interact with this
network.

One of the core principles of Ratman is to make network roaming
easier, building a general abstraction over a network, leaving it up
to drivers to interface with implementation specifics.  This also
means that connections in a Ratman network can be ephemeral and can
sustain long periods of inactivity because of the delay-tolerant
nature of this routing approach.

This software is currently in ALPHA!  Many additional features are
planned such as web-of-trust based routing heuristics, a fully
decentralised journal storage balancing algorithm, and integration
with name query services (such as GNS).

Ratman is developed as part of the [Irdest](https://irde.st) project!

## How to install

It's recommended to install Ratman via a distribution package, if one
is available to you!

[![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)

Alternatively you can build and install Ratman from source.  You need
the following dependencies installed:

 - Rust (rustc `v1.42` or higher)
 - protobuf (with support for `proto3`)
 - pkg-config
 - libsodium
 - llvm
 - clang


Afterwards various ratman targets can be built with `cargo`:

```
$ cargo build --bin=ratmand --features="daemon"
$ cargo build --bin=ratcat --features="utils"
$ cargo build --bin=ratctl --features="utils"
```

## How to use

Ratman includes three binaries: `ratmand`, the stand-alone router
daemon, `ratcat`, a command-line utility to interact with a Ratman
network (analogous to `netcat`), and `ratctl`, a command-line utility
to manage Ratman peer states.  Each is documented in the [user
manual]!

If you are writing an application _specifically_ for Irdest/ Ratman,
you can also check out the [ratman-client] Rust library docs!

[ratman-client]: https://docs.rs/ratman-client
[user manual]: docs.irde.st/user/

## License

Ratman is part of the Irdest project, and licensed under the [GNU
Affero General Public License version 3 or
later](../licenses/agpl-3.0.md).

See the main Irdest repository README for additional permissions
granted by the authors for this code.
