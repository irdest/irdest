{ lib
, stdenv
, yarn2nix-moretea
, fixup_yarn_lock
, yarn
, nodejs
}:

let
  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/.eslintcache" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/dist" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/node_modules" name)
    ;
    src = ../../ratman/dashboard;
  };

  packageJSON = ../../ratman/dashboard/package.json;
  yarnLock = ../../ratman/dashboard/yarn.lock;
  yarnNix = yarn2nix-moretea.mkYarnNix { inherit yarnLock; };

  yarnOfflineCache = yarn2nix-moretea.importOfflineCache yarnNix;

  nodeModules = stdenv.mkDerivation {
    name = "ratman-dashboard-modules";
    dontUnpack = true;

    nativeBuildInputs = [ yarn nodejs ];

    buildPhase = ''
      cp ${packageJSON} ./package.json
      cp ${yarnLock} ./yarn.lock
      chmod u+w yarn.lock

      # Yarn writes temporary files to $HOME. Copied from mkYarnModules.
      export HOME=$NIX_BUILD_TOP/yarn_home

      # Make yarn install packages from our offline cache, not the registry
      yarn config --offline set yarn-offline-mirror ${yarnOfflineCache}

      # Fixup "resolved"-entries in yarn.lock to match our offline cache
      ${fixup_yarn_lock}/bin/fixup_yarn_lock yarn.lock

      yarn install --offline --frozen-lockfile --ignore-scripts --no-progress --non-interactive

      patchShebangs node_modules/
    '';

    installPhase = ''
      mkdir -p $out
      cp -R node_modules $out
    '';
  };

in stdenv.mkDerivation {
  name = "ratman-dashboard";
  src = lib.cleanSourceWith {
    filter = name: type:
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/.eslintcache" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/dist" name) &&
      !(lib.hasPrefix "${toString ../../ratman/dashboard}/node_modules" name)
    ;
    src = ../../ratman/dashboard;
  };

  nativeBuildInputs = [ yarn nodejs ];

  outputs = [ "out" "dist" ];

  buildPhase = ''
    # Yarn writes temporary files to $HOME. Copied from mkYarnModules.
    export HOME=$NIX_BUILD_TOP/yarn_home

    rm -rf node_modules
    ln -s ${nodeModules}/node_modules node_modules

    # Build into `./dist/`, suppress formatting.
    yarn --offline build | cat
  '';

  installPhase = ''
    cp -R . $out

    mv $out/dist $dist
    ln -s $dist $out/dist
  '';
}
