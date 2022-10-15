{ pkgs ? import ../. }:

{
  simple-two-nodes = import ./simple-two-nodes.nix { inherit pkgs; };
}
