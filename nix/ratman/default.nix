{ lib
, callPackage
, rustPlatform
, protobuf
, libsodium
, pkg-config
, ratman-webui
}:

rustPlatform.buildRustPackage rec {
  pname = "ratman";
  version = "development";

  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../.}/docs" name) &&
      !(lib.hasPrefix "${toString ../../.}/target" name) &&
      !(lib.hasPrefix "${toString ../../.}/nix" name) &&
      !(lib.hasPrefix "${toString ../../.}/ratman/webui" name)
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

  # Pre-build and patch in Web UI frontend assets.
  ratman_webui = ratman-webui;
  preBuild = ''
    ln -snf $ratman_webui ratman/webui
  '';

  cargoLock.lockFile = ../../Cargo.lock;
}
