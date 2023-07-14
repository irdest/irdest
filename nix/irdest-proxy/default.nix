{ lib
, rustPlatform
, libsodium
, libudev-zero
, pkg-config
, protobuf
, sqlite
}:

rustPlatform.buildRustPackage rec {
  pname = "irdest-proxy";
  version = "development";

  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../.}/docs" name) &&
      !(lib.hasPrefix "${toString ../../.}/target" name) &&
      !(lib.hasPrefix "${toString ../../.}/nix" name)
    ;
    src = ../../.;
  };

  nativeBuildInputs = [
    protobuf
    pkg-config
  ];

  cargoBuildFlags = [ "--all-features" "-p" "irdest-proxy" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [
    sqlite

    # This is needed because irdest-proxy relies on ratmand for its
    # tests (I think ?? TODO: can we solve this in better?)
    libudev-zero
  ];

  SODIUM_USE_PKG_CONFIG = 1;

  cargoLock.lockFile = ../../Cargo.lock;
}
