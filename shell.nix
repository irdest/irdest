/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 */

with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "irdest-base";
  buildInputs = [
    rustc cargo rustfmt rust-analyzer clangStdenv
    pkg-config protobuf 
    cargo-watch binutils yarn reuse jq
  ]
  # Special dependencies for macOS builds
  ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}
