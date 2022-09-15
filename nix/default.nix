let
  sources = import ./sources.nix;

  overlay = self: super: {
    irdest-installer = self.callPackage ./irdest-installer {};
    irdest-manual-developer = self.callPackage ./irdest-manual/developer.nix {};
    irdest-manual-user = self.callPackage ./irdest-manual/user.nix {};
    irdest-proxy = self.callPackage ./irdest-proxy {};
    irdest-website = self.callPackage ./irdest-website {};

    ratman = self.callPackage ./ratman {};
    ratman-dashboard = self.callPackage ./ratman-dashboard {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
