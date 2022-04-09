# SPDX-FileCopyrightText: 2020-2021 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: GPL-3.0-or-later

/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 */

with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "irdest-base";
  buildInputs = [ 
    # core build tools
    rustc cargo rustfmt rust-analyzer clangStdenv

    # core native deps
    pkg-config protobuf

    # for various tests
    cargo-watch binutils reuse jq

    # for the ratman dashboard
    nodejs yarn
  ]
  # Special dependencies for macOS builds
  ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}
