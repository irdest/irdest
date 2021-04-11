{ lib
, stdenv
, graphviz
, mdbook

, production ? true
}:

stdenv.mkDerivation rec {
  pname = "irdest-docs-developer";
  version = "0.0.0";
  src = lib.cleanSourceWith {
    filter = lib.cleanSourceFilter;
    src = lib.cleanSourceWith {
      filter = name: type:
        name == "${toString ../../.}" ||
        name == "${toString ../../.}/docs" ||
        lib.hasPrefix "${toString ../../.}/docs/developer" name ||
        lib.hasPrefix "${toString ../../.}/licenses" name
      ;
      src = ../../.;
    };
  };

  nativeBuildInputs = [
    mdbook
    graphviz
  ];

  buildPhase = ''
    mkdir $out
    cd docs/developer
    mdbook build -d $out
  '';

  dontInstall = true;
}
