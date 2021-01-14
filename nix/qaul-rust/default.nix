{ lib
, stdenv
, naersk
, cmake
, buildPackages
}:

naersk.buildPackage {
  src = lib.cleanSource ../../.;
  nativeBuildInputs = [
    cmake
  ];
}
