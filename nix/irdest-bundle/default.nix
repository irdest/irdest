{ lib
, stdenv
, ratman
, irdest-proxy
, irdest-installer
, irdest-manual-user
}:

stdenv.mkDerivation rec {
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

  nativebuildInputs = [
    ratman irdest-proxy irdest-installer irdest-manual-user
  ];

  buildPhase = ''
    echo "Nothing to do :)"
  '';

  installPhase = ''
    # Create some directories first
    mkdir -p $out/{bin,dist,man}

    # Copy other Nix outputs
    cp ${ratman}/bin/* $out/bin/
    cp ${irdest-proxy}/bin/* $out/bin/
    cp ${irdest-installer}/bin/* $out/
    cp -r ${irdest-manual-user} $out/manual

    # Copy files from source
    cp README.md $out/
    cp dist/* $out/dist/
    cp docs/man/* $out/man/
    ln -s $out/manual/index.html $out/README.html
  '';
}
