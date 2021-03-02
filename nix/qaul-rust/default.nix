{ lib
, stdenv
, naersk
, buildPackages
, atk
, capnproto
, cmake
, glib
, gtk3
, gtk3-x11
, libsodium
, pkg-config
}:

naersk.buildPackage rec {
  src = lib.cleanSourceWith {
    filter = lib.cleanSourceFilter;
    src = lib.cleanSourceWith {
      filter = name: type:
        !(lib.hasPrefix "${toString ../../.}/emberweb" name) &&
        !(lib.hasPrefix "${toString ../../.}/docs" name) &&
        !(lib.hasPrefix "${toString ../../.}/target" name) &&
        !(lib.hasPrefix "${toString ../../.}/nix" name)
      ;
      src = ../../.;
    };
  };

  nativeBuildInputs = [
    capnproto
    cmake
    pkg-config

    # Required for qaul-gtk
    glib
    gtk3
    atk
    gtk3-x11
  ];

  buildInputs = [
    libsodium
  ];

  SODIUM_USE_PKG_CONFIG = 1;

  doDoc = true;
  doDocFail = true;
  cargoDocOptions = (x: x ++ [ "--no-deps" ]);

  passthru.testBinaries = naersk.buildPackage {
    inherit src nativeBuildInputs buildInputs SODIUM_USE_PKG_CONFIG;

    cargoBuildOptions = (x: x ++ [ "--tests" "-p alexandria2" ]);

    release = false;

    installPhase = ''
      find target/debug/deps -type f -executable -regex ".*-[0-9a-f]+" \
        | sed 's#\(.*\)/\([^/]*\)-\([0-9a-f]\+\)#install -Dm755 \1/\2-\3 \$out/bin/\2-test#' \
        | sh
    '';
  };
}
