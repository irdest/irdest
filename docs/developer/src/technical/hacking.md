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
$ cargo build -p ratman --release --disable-default-features \
                        --features "cli datalink inet lan lora upnp"
...
[cargo goes brrr]
```


## Building irdest-echo

`irdest-echo` is a demo application built specifically to work with
Ratman as a networking backend.  Build it via the `irdest-echo`
package with cargo.

```console
$ cargo build -p irdest-echo --release
...
```


## Building irdest-mblog

`irdest-mblog` is probably the most complete user-facing application
that is native to the Irdest network.  You can build it with Cargo, as
long as you have `gtk4` installed on your system (or using the Nix
environment).

```console
$ cd client/irdest-mblog
$ cargo build --release --bin irdest-mblog-gtk --features "mblog-gtk"
...
```

## What now?

Check the issue tracker for ["good first
issues"](https://git.irde.st/we/irdest/-/issues/?sort=created_date&state=opened&label_name%5B%5D=L%3A%20good%20first%20issue&first_page_size=20)
if you are completely new to Irdest, and additionally ["help
wanted"](https://git.irde.st/we/irdest/-/issues/?sort=created_date&state=opened&label_name%5B%5D=L%3A%20help%20wanted&first_page_size=20)
issues if you already have some experience with the code-base.

Please also don't hesitate to ask us any questions!  We're very happy
to help :)
