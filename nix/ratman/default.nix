{ lib
, rustPlatform
, protobuf
, libsodium
, pkg-config
}:

rustPlatform.buildRustPackage rec {
  pname = "ratman";
  version = "development";

  src = lib.cleanSource ../..;

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
