with import <nixpkgs> {
  config.android_sdk.accept_license = true;
  config.allowUnfree = true;
};

stdenv.mkDerivation {
  name = "qaul";
  buildInputs = with pkgs; [

    # General rust stuff
    rustc cargo rustfmt rust-analyzer

    clangStdenv cargo-watch binutils
    
    # Required for the docs
    mdbook graphviz rsync

    # Required for Android integration
    cmake

    # Required for libqaul-voice
    pkg-config llvm clang 
    llvmPackages.clang-unwrapped

    # webgui debugging and development
    httpie nodejs yarn

    # Required for the code coverage and stuff
    openssl

    # Required for the RPC protocol
    capnproto
    
  ] ++ (with androidenv.androidPkgs_9_0; [
    # Required for Android builds
    androidsdk
    build-tools
    ndk-bundle
    platform-tools

    pkgs.openjdk
  ]) ++ lib.optionals stdenv.isDarwin [
    # Used to build on MacOS
    darwin.apple_sdk.frameworks.Security
  ];
}
