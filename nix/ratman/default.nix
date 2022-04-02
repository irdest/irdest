{ lib
, rustPlatform
, protobuf
, libsodium
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
  pname = "ratman";
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

  cargoBuildFlags = [ "--all-features" "-p" "ratman" ];
  cargoTestFlags = cargoBuildFlags;

  buildInputs = [
    libsodium
  ];

  SODIUM_USE_PKG_CONFIG = 1;

  # Pre-build and patch in Web UI frontend assets.
  ratman_dashboard = ratman-dashboard;
  preBuild = ''
    ln -snf $ratman_dashboard ratman/dashboard
  '';

  cargoLock.lockFile = ../../Cargo.lock;
}
