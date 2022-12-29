{ stdenv, lib, irdest-installer, irdest-manual-user, ratman }:

stdenv.mkDerivation {
  pname = "irdest-bundle";
  version = "0.0.0";

  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../.}/target" name) &&
      !(lib.hasPrefix "${toString ../../.}/nix" name) &&
      !(lib.hasPrefix "${toString ../../.}/ratman/dashboard" name)
    ;
    src = ../../.;
  };

  doBuild = false;

  installPhase = ''
    mkdir $out
    cp -rv dist $out/dist
    cp -rv docs/man $out/man
    cp -rv ${ratman}/bin $out/bin
    cp -rv ${irdest-installer}/bin/* $out
    cp -rv ${irdest-manual-user} $out/manual
  '';
}
