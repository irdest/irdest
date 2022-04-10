# Hacking on Irdest

Hey, it's cool that you want to hack on Irdest :) We recommend you
install [nix](https://nixos.org) to handle dependencies.  Depending on
the directory you are in you can fetch development dependencies:

```console
$ cd irdest/
$ nix-shell # install the base dependencies
...
$ cd docs/
$ nix-shell # install documentation dependencies
```

With [lorri] and [direnv] installed transitioning from one directory
to another will automatically load additional dependencies!

[lorri]: https://github.com/target/lorri
[direnv]: https://direnv.net/

Alternatively, make sure you have the following dependencies
installed:

- rustc
- cargo
- rustfmt
- rust-analyzer
- clangStdenv
- pkg-config
- protobuf 
- cargo-watch
- binutils
- yarn
- reuse
- jq


## Building Ratman

Ratman provides several binaries in the `ratman` package.  You can
build the entire package with `cargo`.  By default the
ratman-dashboard will be included, which requires you to build the
sources with `yarn` first.

```console
$ cd ratman/dashboard
$ yarn && yarn build
$ cd ../..
$ cargo build -p ratman --all-features
```

Alternatively you can disable the `dashboard` feature.  Unfortunately
`cargo` doesn't allow selective disabling of features, so you will
need to disable all default features, then select a new set of
features as follows:

```console
$ cargo build -p ratman --release --disable-default-features --features "cli inet lan upnp"
...
```


## Building irdest-echo

`irdest-echo` is a demo application built specifically to work with
Ratman as a networking backend.  Build it via the `irdest-echo`
package with cargo.

```console
$ cargo build -p irdest-echo --release
...
```
