with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "irdest-docs";
  buildInputs = with pkgs; [
    mdbook hugo
    mdbook-graphviz graphviz
    mdbook-mermaid
  ];
}
