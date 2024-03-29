{ lib
, rustPlatform
, protobuf
, libsodium
, pkg-config
, gtk4
, glib
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
    glib
    pkg-config
  ];

  cargoBuildFlags = [ "--all-features" "-p" "irdest-mblog" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [ gtk4 ];

  cargoLock.lockFile = ../../Cargo.lock;
}
