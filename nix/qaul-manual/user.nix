{ lib
, stdenv
, graphviz
, mdbook

, production ? true
}:

stdenv.mkDerivation rec {
  pname = "qaul-manual-user";
  version = "0.0.0";
  src = lib.cleanSourceWith {
    filter = lib.cleanSourceFilter;
    src = lib.cleanSourceWith {
      filter = name: type:
        name == "${toString ../../.}" ||
        name == "${toString ../../.}/docs" ||
        lib.hasPrefix "${toString ../../.}/docs/user" name ||
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
    cd docs/user
    mdbook build -d $out
  '';

  dontInstall = true;
}
