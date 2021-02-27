let
  sources = import ./sources.nix;

  overlay = self: super: {
    naersk = self.callPackage sources.naersk {};
    qaul-docs = self.callPackage ./qaul-docs {};
    qaul-rust = self.callPackage ./qaul-rust {};
    qaul-web = self.callPackage ./qaul-web {};
    qaul-website = self.callPackage ./qaul-website {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
