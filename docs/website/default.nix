with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "irdest-website";
  buildInputs = with pkgs; [
    hugo
  ];
}
