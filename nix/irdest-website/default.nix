{ stdenv
, hugo
, rsync
}:

let
  npm2nix = import (builtins.fetchGit {
    url = "https://github.com/nix-community/npmlock2nix";
    rev = "5c4f247688fc91d665df65f71c81e0726621aaa8";
  }) {};
in
npm2nix.build {
  src = ../../docs/website-new;
  installPhase = "mkdir $out && cp -r dist $out";
  buildCommands = [ "env XDG_CONFIG_HOME=.tmp npm run build" ];
}
