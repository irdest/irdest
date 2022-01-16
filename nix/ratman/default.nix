{ lib
, rustPlatform
, protobuf
, libsodium
, pkg-config
}:

rustPlatform.buildRustPackage rec {
  pname = "ratman";
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

  cargoBuildFlags = [ "--all-features" "-p" "ratman" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [
    libsodium
  ];

  SODIUM_USE_PKG_CONFIG = 1;

  cargoLock.lockFile = ../../Cargo.lock;
}
