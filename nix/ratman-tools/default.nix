# SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

{ lib
, rustPlatform
, protobuf
, pkg-config
}:

let
  # Build the dashboard assets natively on the builder, even when cross-compiling.
  #
  # This works around an issue where eg. `pkgsStatic.yarn2nix` fails to evaluate.
  # See: https://github.com/NixOS/nixpkgs/issues/116207
  inherit (import ../.) ratman-dashboard;
in

rustPlatform.buildRustPackage rec {
  pname = "ratman-tools";
  version = "development";

  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../.}/docs" name) &&
      !(lib.hasPrefix "${toString ../../.}/target" name) &&
      !(lib.hasPrefix "${toString ../../.}/nix" name) &&
      !(lib.hasPrefix "${toString ../../.}/ratman/dashboard" name)
    ;
    src = ../../.;
  };

  nativeBuildInputs = [
    protobuf
    pkg-config
  ];

  cargoBuildFlags = [ "--all-features" "-p" "ratman-tools" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [ ];

  cargoLock.lockFile = ../../Cargo.lock;
}
