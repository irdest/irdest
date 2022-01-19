---
Title: Downloads
layout: page
---

Before downloading, please beware that this is **alpha stage
software** and as such it will have bugs and produce crashes.  Please
do not expect to be able to rely on this software in production
setups.

That being said: we want to improve Irdest for everyone so if you
experience a crash, please report the issue to our [issue
tracker][issues] or our [community mailing ist][mail]!

[issues]: https://git.irde.st/we/irdest/-/issues
[mail]: https://lists.irde.st/archives/list/community@lists.irde.st/


<img src="/img/ratman-banner.png" width="800px" />

The main way to use Irdest at the moment is by installing and
configuring the *Ratman* decentralised peer-to-peer routing daemon.


### Distribution packages

You can find pre-made packages for several Linux and BSD
distributions.  Consult the following table for details.

[![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)


### Static/ portable binaries

As part of our CI pipeline we build static binaries for Ratman and
associated tools.  You can grab the latest successful build of the
Ratman release branch below

- Ratman [x86_64 binaries](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.3.0/raw/ratman-bundle-x86_64.tar.gz?job=bundle-ratman)!
- Ratman [aarch64 binaries](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.3.0/raw/ratman-bundle-aarch64.tar.gz?job=bundle-ratman-aarch64)


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
   
   - rustc (you need rustc `v1.55` or higher)
   - protobuf (with support for `proto3`)
   - pkg-config
   - libsodium
   - llvm
   - clang

3. Now you can build the `ratmand` target with `cargo`:

   ```console
   $ cargo build --bin ratmand --features "daemon" --release
   $ ./target/release/ratmand
   ```

Congratulations!  Now consult the [user
manual](https://docs.irde.st/user/) on how to configure the daemon!
