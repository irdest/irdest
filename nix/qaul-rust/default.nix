{ lib
, stdenv
, naersk
, cmake
, pkg-config
, libsodium
, buildPackages
}:

naersk.buildPackage {
  src = lib.cleanSource ../../.;
  nativeBuildInputs = [
    cmake
    pkg-config
  ];

  buildInputs = [
    libsodium
  ];

  SODIUM_USE_PKG_CONFIG = 1;
}
