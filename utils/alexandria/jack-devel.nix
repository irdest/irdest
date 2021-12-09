with import <nixpkgs> {};

let
  my_jack = jack.overrideAttrs ({ ... }: {
    src = /home/lauren/projects/jack;
  });
in
stdenv.mkDerivation {
  name = "teufel-secret-sauce";
  src = /home/lauren/teufel/secret-sauce;

  nativeBuildInputs = [ cmake ];

  buildInputs = [ my_jack ];
}
