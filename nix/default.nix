let
  sources = import ./sources.nix;

  overlay = self: super: {
    irdest-manual-developer = self.callPackage ./irdest-manual/developer.nix {};
    irdest-manual-user = self.callPackage ./irdest-manual/user.nix {};
    irdest-website = self.callPackage ./irdest-website {};
    ratman = self.callPackage ./ratman {};
    ratman-webui = self.callPackage ./ratman-webui {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
