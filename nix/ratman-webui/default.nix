{ lib
, mkYarnPackage
}:

mkYarnPackage {
  name = "ratman-webui";
  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../ratman/webui}/.eslintcache" name) &&
      !(lib.hasPrefix "${toString ../../ratman/webui}/dist" name) &&
      !(lib.hasPrefix "${toString ../../ratman/webui}/node_modules" name)
    ;
    src = ../../ratman/webui;
  };

  outputs = [ "out" "dist" ];

  buildPhase = ''
    # Yarn writes temporary files to $HOME. Copied from mkYarnModules.
    export HOME=$PWD/yarn_home

    # Build into `./dist/`, suppress formatting.
    yarn --offline build | cat
  '';

  installPhase = ''
    cp -R ./deps/webui $out
    ln -snf $node_modules $out/node_modules

    mv $out/dist $dist
    ln -s $dist $out/dist

    # Nonsense link created by nix.
    rm $out/webui
  '';

  # Don't generate a dist tarball for npm.
  distPhase = "true";
}
