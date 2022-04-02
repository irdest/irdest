{ lib
, mkYarnPackage
}:

mkYarnPackage {
  name = "ratman-dashboard";
  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/.eslintcache" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/dist" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/node_modules" name)
    ;
    src = ../../ratman/dashboard;
  };

  outputs = [ "out" "dist" ];

  buildPhase = ''
    # Yarn writes temporary files to $HOME. Copied from mkYarnModules.
    export HOME=$PWD/yarn_home

    # Build into `./dist/`, suppress formatting.
    yarn --offline build | cat
  '';

  installPhase = ''
    cp -R ./deps/ratman-dashboard $out
    ln -snf $node_modules $out/node_modules

    mv $out/dist $dist
    ln -s $dist $out/dist

    # Nonsense link created by nix.
    rm $out/ratman-dashboard
  '';

  # Don't generate a dist tarball for npm.
  distPhase = "true";
}
