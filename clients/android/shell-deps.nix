{ pkgs, ... }: with pkgs.androidenv.androidPkgs_9_0; [
  androidsdk
  build-tools
  ndk-bundle
  platform-tools

  pkgs.openjdk
]
