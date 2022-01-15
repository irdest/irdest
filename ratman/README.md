# Ratman

A modular userspace packet router, providing decentralised distance
vector routing and delay tolerance.  Both a Rust library and
stand-alone and zero-config application daemon.

Ratman provides a 256-bit address space via ed25519 public keys.  This
means that messages are end-to-end encrypted and authenticated at the
transport level (not currently implemented!).  Fundamentally Ratman
exists outside the OSI layer scopes.  On one side it binds to
different transport layers (for example IP, or BLE) via "net module"
drivers.  On the other side it provides a simple protobuf API on a
local TCP socket for applications to interact with this network.

One of the core principles of Ratman is to make network roaming
easier, building a general abstraction over a network, leaving it up
to drivers to interface with implementation specifics.  This also
means that connections in a Ratman network can be ephemeral and can
sustain long periods of inactivity because of the delay-tolerant
nature of this routing approach.

This software is currently in ALPHA!


## How to install

It's recommended to install Ratman via a distribution package, if one
is available to you!  Check https://irde.st/downloads for details.

Alternatively you can build and install Ratman from source.  You need
the following dependencies installed:

 - Rust (rustc `v1.42` or higher)
 - protobuf (with support for `proto3`)
 - pkg-config
 - libsodium
 - llvm
 - clang


## How to use

Ratman includes two binaries: `ratmand`, the stand-alone router
daemon, and `ratman-gen`, a command-line utility to interact with the
daemon.  If you are writing an application _specifically_ for Irdest/
Ratman, you can also check out the [ratman-client] Rust library docs!

[ratman-client]: https://docs.rs/ratman-client

## License

Ratman is part of the Irdest project, and licensed under the [GNU
Affero General Public License version 3 or
later](../licenses/agpl-3.0.md).

See the main Irdest repository README for additional permissions
granted by the authors for this code.
g
