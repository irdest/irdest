let
  pkgs = import <nixpkgs> { };
  lib = pkgs.lib;
  sources = import ./sources.nix;
  naersk = pkgs.callPackage sources.naersk { };

in {
  qaul-rust = naersk.buildPackage {
    src = lib.cleanSource ../.;
    singleStep = true;
    nativeBuildInputs = with pkgs; [ cmake ];
  };
}
