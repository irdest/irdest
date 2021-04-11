let
  sources = import ./sources.nix;

  overlay = self: super: {
    naersk = self.callPackage sources.naersk {};
    irdest-manual-developer = self.callPackage ./irdest-manual/developer.nix {};
    irdest-manual-user = self.callPackage ./irdest-manual/user.nix {};
    irdest-rust = self.callPackage ./irdest-rust {};
    irdest-website = self.callPackage ./irdest-website {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
