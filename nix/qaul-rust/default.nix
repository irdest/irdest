{ lib
, stdenv
, naersk
, capnproto
, cmake
, pkg-config
, libsodium
, buildPackages
}:

naersk.buildPackage {
  src = lib.cleanSource ../../.;
  nativeBuildInputs = [
    capnproto
    cmake
    pkg-config
  ];

  buildInputs = [
    libsodium
  ];

  SODIUM_USE_PKG_CONFIG = 1;
}
