with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "irdest-docs";
  buildInputs = with pkgs; [ mdbook graphviz hugo ];
}
