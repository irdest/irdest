{ lib
, stdenv
, mkYarnModules
, yarn
, rsync
, nodejs

, production ? true
}:

let
  src = lib.cleanSource ../../emberweb;

  packageJSON = ../../emberweb/package.json;
  yarnLock = ../../emberweb/yarn.lock;

  package = lib.importJSON packageJSON;
  pname = package.name;
  version = package.version;

  modules = mkYarnModules {
    pname = "${pname}-modules";
    name = "${pname}-modules-${version}";
    inherit version packageJSON yarnLock;
  };

in stdenv.mkDerivation rec {
  inherit src pname version;

  nativeBuildInputs = [
    yarn
    rsync
    nodejs
  ];

  configurePhase = ''
    runHook preConfigure
    export HOME=$NIX_BUILD_TOP/fake_home
    ln -sf ${modules}/node_modules ./node_modules
    runHook postConfigure
  '';

  buildPhase = ''
    runHook preBuild
    yarn --offline run build${lib.optionalString production " --prod"}
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    rsync -a dist/ $out/
    runHook postInstall
  '';
}
