{ lib
, rustPlatform
, protobuf
, libsodium
, pkg-config
, gtk4
}:

rustPlatform.buildRustPackage rec {
  pname = "irdest-mblog";
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

  cargoBuildFlags = [ "--all-features" "-p" "irdest-mblog" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [
    libsodium
    gtk4
  ];

  SODIUM_USE_PKG_CONFIG = 1;

  cargoLock.lockFile = ../../Cargo.lock;
}
