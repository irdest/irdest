![](../docs/ratman-banner.png)

The main piece of software created by the Irdest project.

Ratman is a modular userspace packet router.  It uses "distance vector
routing" to move data across a decentralised network.  The topology of
this network is unknowable for any individual user of the network.  In
the future Ratman will also handle delay-tolerant connections, going
days, weeks, or even months between direct connections between two
users on the network.

Addresses in Ratman are ed25519 public keys, and thus create a 256-bit
address space.  This is 8x langer than IPv6 and means that messages
are by default end-to-end encrypted and authenticated at the transport
level (although this currently isn't enforced).  Addresses in this
network are automatically announced and propagated so that every user
in the network can send messages to any other user, without exactly
knowing how to get there.

Fundamentally Ratman exists outside the OSI layer scopes.  On one side
it binds to different transport layers (for example IP, or BLE) via
"netmod" drivers.  On the other side it provides a protobuf API on a
local TCP socket for applications to interact with this network.

One of the core principles of Ratman is to make network roaming
easier, building a general abstraction over a network, leaving it up
to drivers to interface with implementation specifics.  This also
means that connections in a Ratman network can be ephemeral and can
sustain long periods of inactivity because of the delay-tolerant
nature of this routing approach.

**This software is currently in ALPHA!**  Many additional features are
planned such as web-of-trust based routing heuristics, a fully
decentralised journal storage, and integration with name query
services such as the GNS.

Ratman is developed as part of the [Irdest](https://irde.st) project!


## How to install

It's recommended to install Ratman via a distribution package, if one
is available to you!

[![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)

If no package is available to you, but you are running on an otherwise
supported platform (x86_64 linux and arm64 linux), we do have static
binaries available via out CI bundles.  They also include an installer
and instructions on how to setup Ratman.  You can find them on the
[download page](https://irde.st/downloads).  If you are running macOS,
you can compile Ratman from source (see below).  Testing and
[reporting issues]() is highly appreciated!

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


## Hacking on Ratman

We recommend you use the [nix file](../shell.nix) to load required
dependencies into your shell.  Afterwards you can simply use `cargo`
to build Ratman and other tools in this repository.

```console
$ nix-shell
...
$ cargo build --bin ratmand
```

If you want to build Ratman with the web dashboard, you will first
have to follow the steps outlined in
[ratman/dashboard](./dashboard/README.md).


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
