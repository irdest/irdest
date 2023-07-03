# SPDX-FileCopyrightText: 2020-2021 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: GPL-3.0-or-later

/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 */

with import (import nix/sources.nix).nixpkgs {};

stdenv.mkDerivation {
  name = "irdest-base";
  buildInputs = [ 
    # core build tools
    rustc cargo rustfmt rust-analyzer clangStdenv

    # core native deps
    pkg-config protobuf udev

    # SQL migration tool
    sqlx-cli

    # for various tests
    cargo-watch binutils reuse jq

    # ratman dashboard + benchmark graphs
    nodejs yarn gnuplot

    # for irdest-mblog
    gtk4
  ]
  # Special dependencies for macOS builds
  ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}
