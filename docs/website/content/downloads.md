---
Title: Downloads
layout: page
---

Before downloading, please beware that this is **alpha stage
software** and as such it will have bugs and produce crashes.  Please
do not expect to be able to rely on this software in production
setups.

That being said: we want to improve Irdest for everyone so if you
experience a crash, please report the issue to our issue tracker or
our [community mailing
ist](https://lists.irde.st/archives/list/community@lists.irde.st/)


## Ratman

The main way to use Irdest at the moment is by installing and
configuring the *Ratman* decentralised peer-to-peer routing daemon.

### Distribution packages

You can find pre-made packages for several Linux and BSD
distributions.  Consult the following table for details.

[![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)

### Building from source

If your platform does not yet have a package, you can build Ratman
from source.


1. Clone https://git.irde.st/we/irdest with git
   
   ```console
   $ git clone https://git.irde.st/we/irdest
   $ cd irdest
   ```

2. If you have [nix](https://nixos.org/) installed on your system you
   can fetch all dependencies by running `nix-shell`.  Otherwise install
   these dependencies manually:
   
   - rustup (you need rustc `v1.42` or higher)
   - libsodium
   - protobuf (with support for `proto3`)
   - pkg-config
   - llvm
   - clang

3. Now you can build the `ratmand` target with `cargo`:

   ```console
   $ cargo build --bin ratmand --release
   $ ./target/release/ratmand
   ```

Congratulations!  Now consult the [users manual](/learn#manuals) on
how to configure the daemon!
