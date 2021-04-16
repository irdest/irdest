# Nix builds

While it is possible to install dependencies with platform specific
tools (such as `apt` on Debian, etc), it is far more recommended to
use [nix](https://nixos.org) to build irdest instead.

Follow the instructions on how to install nix on your platform
[here][nix-instructions]

[nix-instructions]: https://nixos.org/download.html

## Fetch dependencies

The `shell.nix` in the irdest repo root defines dependencies.  Fetch
them into your environment by running `nix-shell` in the repo root
(this might take a while).

Afterwards you can simple run `cargo build --bin irdest-hubd
--release` to build a new hubd binary.

The output artifact will be written to `./target/release/irdest-hubd`.
