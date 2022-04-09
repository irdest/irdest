<div align="center">
    <img src="docs/banner.png" height="200px"/>
</div>

---

[![Built with Nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

**Irdest** is a decentralised networking project, aiming to create
easy-to-use solutions for ad-hoc wireless, and mesh networks.  It
supports many common desktop operating systems (Linux, Windows, MacOS,
NetBSD, …), and Android mobile phones.  iOS support is on the roadmap.

The core component of the Irdest project is **Ratman**, a
decentralised, peer-to-peer packet router (following the [gossip
protocol] approach), written in [Rust].  With Ratman you can create
private overlay networks (similar to VPNs), connect to a wider
community mesh of existing overlay networks, plug into specific
wireless drivers for fully off-the-grid routing, or do all of them at
the same time.

Following is an overview of the available components.  Components have
additional information in their respective README files.

| Component  | Description                                                                                                                                |
|------------|--------------------------------------------------------------------------------------------------------------------------------------------|
| [clients]  | End-user applications using Irdest.  These include both shims to traditional networking, as well as apps written specificically for Irdest |
| [docs]     | Manuals (for both users and developers), and tools to build and deploy documentation                                                       |
| [licenses] | Set of license texts that are in use in this repository                                                                                    |
| [netmods]  | Platform-specific networking interface drivers                                                                                             |
| [nix]      | [Nix](https://nixos.org) related build utilities                                                                                           |
| [ratman]   | A decentralised and modular userspace frame router                                                                                         |
| [tests]    | Integration tests over multiple components
| [utils]    | Various utilities in use all over the repository that don't fit in anywhere else                                                           |                                                                                                                                           |


## Overview

Most traditional networking infrastructure (both the transmission
layer, as well as applications) operate in a centralised way.  Clients
connect to servers, and devices to [ISP]s.  This makes the
infrastructure vulnerable to attacks.  Natural disasters or opressive
governments can easily shut down communication for millions of people,
potentially putting them at risk, and slowing down or disrupting any
organisation or activist movement.

Irdest aims to solve this issue by creating decentralised circuits
between devices directly.  These direct device-to-device connections
can be imperfect and unstable.  Irdest's routing approach takes these
issues into account by caching undelivered messages, and carrying them
towards their destination until the receipient comes back online.

Routing in an Irdest network is done via a user's **ed25519 public
keys**, creating a 32 byte large address space (twice as lange as
IPv6).  Connecting devices together happens via channel-specific
drivers (for example the tcp internet overlay). Therefore when
creating a circuit, roaming between different connection types is
normal, and no single connection technology has to work on all
possible devices or platforms.

To learn more about the technical side of Irdest, check out the
[developer manual].


## How to use

The easiest way to get started using Irdest is to download
[Ratman][Downloads] and its associated utilities.  To learn how, check
out the [user manual]!


## Contributing

To get started to hack on Irdest, check out the [HACKING section] in
the developer manual.

Social processes, code, and design guidelines are outlined in the
[developer manual].  We have a developer chat hosted on [Matrix] where
we would be happy to answer any questions you have.

If you want some inspiration for what you can do with irdest, check
out the [clients] section.


## License

Irdest is free software licensed under the [GNU Affero General Public
License version 3](licenses/agpl-3.0.md) or later.

**Additional Permissions:** For Submission to the Apple App Store:

Provided that you are otherwise in compliance with the AGPLv3 for each
covered work you convey (including without limitation making the
Corresponding Source available in compliance with Section 6 of the
AGPLv3), the irdest developers also grant you the additional
permission to convey through the Apple App Store non-source executable
versions of the Program as incorporated into each applicable covered
work as Executable Versions only under the Mozilla Public License
version 2.0.

A copy of both the AGPL-3.0 and MPL-2.0 license texts are included in
this repository, along other external licenses for third-party code,
and can be found in the [licenses](licenses) directory.


[Matrix]: https://matrix.to/#/#irdest:fairydust.space?via=ontheblueplanet.com&via=matrix.org&via=fairydust.space
[Downloads]: https://irde.st/downloads
[Rust]: https://rust-lang.org
[ISP]: https://en.wikipedia.org/wiki/ISP
[clients]: ./clients
[developer manual]: https://docs.irde.st/developer/
[docs]: ./docs
[gossip protocol]: https://en.wikipedia.org/wiki/Gossip_protocol
[HACKING section]: https://docs.irde.st/developer/technical/hacking.html
[irdest-mblog]: mblog.irde.st
[licenses]: ./licenses
[netmods]: ./netmods
[nix]: ./nix
[ratman]: ./ratman
[tests]: ./tests
[user manual]: https://docs.irde.st/user/
[utils]: ./utils
