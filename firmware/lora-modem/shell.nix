# SPDX-FileCopyrightText: 2020-2021 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: GPL-3.0-or-later

/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 */

with import <nixpkgs> {
  crossSystem = (import <nixpkgs/lib>).systems.examples.armhf-embedded // {
    rustc.config = "thumbv7em-none-eabihf";
  };
  overlays = [
    (self: super: {
      rust_1_62 = super.rust_1_62 // {
        packages = super.rust_1_62.packages // {
          stable = super.rust_1_62.packages.stable.overrideScope' (self: super: {
            rustc = super.rustc.overrideAttrs ({ configureFlags ? [], ... }: {
              configureFlags = configureFlags ++ [ "--disable-docs" ];
            });
          });
        };
      };
    })
  ];
};
stdenv.mkDerivation {
  name = "irdest-firmware";
  nativeBuildInputs = with pkgsBuildHost; [
    rustc cargo cargo-binutils lld clangStdenv

    gdb openocd
  ];
}
