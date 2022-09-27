let
  sources = import ./sources.nix;

  overlay = self: super: {
    irdest-website = self.callPackage ./irdest-website {};
    irdest-manual-developer = self.callPackage ./irdest-manual/developer.nix {};
    irdest-manual-user = self.callPackage ./irdest-manual/user.nix {};
    
    irdest-installer = self.callPackage ./irdest-installer {};
    irdest-proxy = self.callPackage ./irdest-proxy {};
    
    ratman = self.callPackage ./ratman {};
    ratman-dashboard = self.callPackage ./ratman-dashboard {};

    irdest-bundle = self.callPackage ./irdest-bundle {};
  };

in
  import sources.nixpkgs {
    overlays = [ overlay ];
  }
