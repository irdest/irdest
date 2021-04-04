---
Title: Downloads
layout: page
---

Most user-facing irdest applications and tools are still very
work-in-progress.  Target platforms include Linux, MacOS, Windows,
Android, and iOS.  However currently only Linux and Android are
supported!

Following are instructions on how to build the main irdest router
daemon on Linux, which enables you to join a irdest network over the
internet.


## irdest-hubd


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
   - pkg-config
   - llvm
   - clang

3. Now you can build the `irdest-hubd` target with `cargo`:

   ```console
   $ cargo build --bin irdest-hubd --release
   $ ./target/release/irdest-hubd
   ```

Congratulations!  Now consult the [users manual](/learn#manuals) on
how to configure the daemon!


## irdestdroid

The irdest Android app is currently still a prototype and not
intended for end-users.  Building it requires a full Android
development setup installed on your system.  To make the Rust
cross-compilation easier, we created a [docker build
environment][docker]!

[docker]: https://hub.docker.com/r/qaulnet/android-build-env


1. Clone the main irdest repo as before:

   ```console
   $ git clone https://git.irde.st/we/irdest
   $ cd irdest/clients/android
   ```
   
2. Cross-compile the Rust libraries via docker:

   ```console
   $ ./build.sh
   ```
   
   This will take a while!
   
3. Now you can build the main Android application with gradle:

   ```console
   $ ./gradlew dist
   ```
