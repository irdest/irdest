{ stdenv
, hugo
, rsync
}:

stdenv.mkDerivation rec {
  pname = "qaul-website";
  version = "0.0.0";

  src = ../../docs/website;

  buildInputs = [
    hugo
    rsync
  ];

  HUGO_DISABLELANGUAGES = "ar";

  buildPhase = ''
    runHook preBuild
    hugo
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    rsync -a ./public/ $out/
    runHook postInstall
  '';
}
