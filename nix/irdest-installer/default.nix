# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
# SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

{ lib
, rustPlatform
, protobuf
, libsodium
, pkg-config
}:

rustPlatform.buildRustPackage rec {
  pname = "irdest-installer";
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

  cargoBuildFlags = [ "--all-features" "-p" "installer" ];
  cargoTestFlags = cargoBuildFlags;

  cargoLock.lockFile = ../../Cargo.lock;
}
