# Build from source

Generally we recommend you install Irdest through your platform's
package manager (if a package is available), or via our [stand-alone
installer][website-installer].

However if no pre-built binaries exist for your platform you can use
these instructions to build Irdest from source!  However **please also
come by our [Matrix room](https://irde.st/community/)** and tell us what
platform you are using Irdest on!  We may be able to build binaries for
you as part of our CI pipeline.

## Download latest source

First you will need to download the latest sources for Irdest.  You
can find a link to the last release [on the website][website-sources],
or via the [Gitlab repository][repo-sources]!

In case you have `git` installed on your system and would like to
follow along with the development version you can clone the repository
as follows:

```console
~ $ git clone https://git.irde.st/we/irdest
~ $ cd irdest/
```

Optionally you can only clone a particular release branch (default
branch is `develop`):

```console
git clone https://git.irde.st/we/irdest --branch release/ratman-0.4.0
```

## Installing dependencies

You will need the following dependencies for building Irdest:

- rustc (minimum version ?)
- cargo
- pkg-config
- protobuf-compiler (with support for `proto3`)
- libudev
- llvm
- clang
  
On macOS you will need the Security framework from the Apple SDK.

**Commonly these packagse can be installed as follows.**  Note that
there is a wide range of operating system setups and so these
instructions can not be comprehensive!

- Ubuntu: `sudo apt install rustc cargo pkg-config protobuf-compiler clang-dev llvm-dev libudev-dev`
- Arch: `sudo pacman -Sy rustc cargo pkg-config protobuf3 clang llvm libudev`

## Nix builds

[Nix](https://nixos.org) is a package manager and build tool which is
used during Irdest development and CI.  It is easy for documentation
to go out of date and forget to mention a dependency or update a
package name.  **Since we use Nix builds during development they will
always be up-to-date.**

Installation and usage is a _bit more_ involved than simply installing
packages, but it may be worth it if you plan on following along
development regularly, or embedding the update process into other
automation!

Follow the instructions on how to install nix on your platform
[here][nix-instructions].

After installation you should be able to build the Irdest bundle as
follows:

```console
nix-build nix/ -A irdest-bundle
result/install  # then run the installer
```

You can also fetch development dependencies by running `nix-shell` in
the Irdest repository root!

## Cargo builds

Since Irdest is written in Rust, you can compile it with Cargo!  Once
you have all required dependencies installed you can invoke cargo as
follows

### Build the dashboard

- Go into the `dashboard` directory

    ```console
    cd ratman/dashaboard
    ```

- Run yarn

    ```console
    yarn build
    ```

- Run `cargo`

    ```console
    cargo release -p ratman --release
    ```

Other buildable targets or features not enabled by default can be
found in the [build reference].

## Firmware

Note that Irdest contains some components that require embedded
firmware.  This can be found in the `firmware` directory in the Irdest
repository.

**Cross-compilation is extremely difficult!**  There are `shell.nix`
files in each of the firmware sub-projects that configure and install
a cross-compilation build environment.

**We strongly recommend using Nix to compile and flash your own
firmware.**  We unfortunately can't give support for setting up
cross-compilation bulid environments on other systems, since they are
so numerous and complicated to configure!

[website-installer]: https://irde.st/download#portable-stand-alone-binaries
[website-sources]: https://irde.st/downloads#sources
[repo-sources]: https://git.irde.st/we/irdest/-/releases
[nix-instructions]: https://nixos.org/download.html
