let
  sources = import ./sources.nix;

  overlay = self: super: {
    irdest-installer = self.callPackage ./irdest-installer {};
    irdest-manual-developer = self.callPackage ./irdest-manual/developer.nix {};
    irdest-manual-user = self.callPackage ./irdest-manual/user.nix {};
    irdest-proxy = self.callPackage ./irdest-proxy {};
    irdest-mblog = self.callPackage ./irdest-mblog {};
    irdest-website = self.callPackage ./irdest-website {};

    irdest-bundle = self.callPackage ./irdest-bundle {
      inherit (self) irdest-installer irdest-manual-user ratman;
    };
    
    ratmand = self.callPackage ./ratmand {};
    ratman-dashboard = self.callPackage ./ratman-dashboard {};
  };

in
import sources.nixpkgs {
  overlays = [ overlay ];
}
