---
Title: Downloads
layout: page
---

Most user-facing qaul.net applications and tools are still very
work-in-progress.  Target platforms include Linux, MacOS, Windows,
Android, and iOS.  However currently only Linux and Android are
supported!

Following are instructions on how to build the main qaul.net router
daemon on Linux, which enables you to join a qaul network over the
internet.


## qaul-hubd


1. Clone https://git.open-communication.net/qaul/qaul.net with git
   
   ```console
   $ git clone https://git.open-communication.net/qaul/qaul.net
   $ cd qaul.net
   ```

2. If you have [nix](https://nixos.org/) installed on your system you
   can fetch all dependencies by running `nix-shell`.  Otherwise install
   these dependencies manually:
   
   - rustup (you need rustc `v1.42` or higher)
   - libsodium
   - pkg-config
   - llvm
   - clang

3. Now you can build the `qaul-hubd` target with `cargo`:

   ```console
   $ cargo build --bin qaul-hubd --release
   $ ./target/release/qaul-hubd
   ```

Congratulations!  Now consult the [users manual](/learn#manuals) on
how to configure the daemon!


## qauldroid

The qaul.net Android app is currently still a prototype and not
intended for end-users.  Building it requires a full Android
development setup installed on your system.  To make the Rust
cross-compilation easier, we created a [docker build
environment][docker]!

[docker]: https://hub.docker.com/r/qaulnet/android-build-env


1. Clone the main qaul.net repo as before:

   ```console
   $ git clone https://git.open-communication.net/qaul/qaul.net
   $ cd qaul.net/clientsl/android
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
