# SPDX-FileCopyrightText: 2020-2021 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: GPL-3.0-or-later

/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 */

with import <nixpkgs> {
  crossSystem = (import <nixpkgs/lib>).systems.examples.armhf-embedded // {
    rustc.config = "thumbv7em-none-eabi";
  };
};
stdenv.mkDerivation {
  name = "irdest-firmware";
  nativeBuildInputs = with pkgsBuildHost; [
    rustc cargo rustfmt rust-analyzer clangStdenv
  ];
}
