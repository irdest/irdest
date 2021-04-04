let
  sources = import ./sources.nix;

  overlay = self: super: {
    naersk = self.callPackage sources.naersk {};
    qaul-manual-developer = self.callPackage ./qaul-manual/developer.nix {};
    qaul-manual-user = self.callPackage ./qaul-manual/user.nix {};
    qaul-rust = self.callPackage ./qaul-rust {};
    qaul-website = self.callPackage ./qaul-website {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
