/** This shell derivation fetches all required dependencies to hack on Irdest
 * 
 * You may want to comment-out the bottom section to ignore Android
 * platform dependencies.
 */

with import <nixpkgs> {
  config.android_sdk.accept_license = true;
  config.allowUnfree = true;
};

let
  inherit (lib.filesystem) listFilesRecursive;
  inherit (builtins) filter toPath toString;
  inherit (lib) flatten hasSuffix;

  ## Each sub-directory project that requires a certain set of
  ## dependencies must declare these in a "shell-deps.nix" file.
  ## These are found by this code and then included.
  findDeps = root:
    map
      (path: toPath path)
      (filter
        (path: hasSuffix "shell-deps.nix" path)
        (map
          (path: toString path)
          (listFilesRecursive root)));
  includeList = list:
    flatten
      (map
        (filePath: (import filePath { inherit pkgs; }))
        list);
in
stdenv.mkDerivation {
  name = "irdest-dev";
  buildInputs = 
    # This set of dependencies should be required by _most_ sub-trees
    # (for example a compiler, pkg-config, etc)
    [
      rustc cargo rustfmt rust-analyzer pkg-config
      clangStdenv cargo-watch binutils
    ]
    # Then include all sub-dependencies
    ++ (includeList (findDeps ./.))
    # We also include some special MacOS things
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
    ];
}
