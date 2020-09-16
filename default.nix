with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "qaul-website";
  buildInputs = with pkgs; [
    hugo
  ];
}
