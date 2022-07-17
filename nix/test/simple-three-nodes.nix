# SPDX-FileCopyrightText: 2022 Yaya <github@uwu.is>
# SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

let
  pkgs = import ../.;

in pkgs.nixosTest {
  name = "ratman-two-nodes";

  nodes =
    let
      mkNode = {vlans, ratmanArgs}: {
        imports = [
          ../nixos-modules/ratman.nix
        ];

        virtualisation = { inherit vlans; };

        environment.systemPackages = with pkgs; [ ratman jq ];

        networking.firewall.allowedTCPPorts = [ 9000 ];

        services.ratman = {
          enable = true;
          extraArgs = [ "-v debug" "--accept-unknown-peers" "--no-discovery" ] ++ ratmanArgs;
        };

        systemd.services = {
          ratcat-register = {
            wantedBy = [ "multi-user.target" ];
            after = [ "ratmand.service" ];
            wants = [ "ratmand.service" ];
            serviceConfig = {
              ExecStart = "${pkgs.ratman}/bin/ratcat --register";
              Type = "oneshot";
              RemainAfterExit = true;
            };
          };
          ratcat-recv = {
            wantedBy = [ "multi-user.target" ];
            after = [ "ratcat-register.service" ];
            wants = [ "ratcat-register.service" ];
            script = ''
              ${pkgs.ratman}/bin/ratcat --recv | ${pkgs.pv}/bin/pv -pb >/dev/null
            '';
          };
        };
      };
  in {
    one = { ... }: mkNode {
      vlans = [ 10 ];
      ratmanArgs = [
        "--peers inet#[fe80::5054:ff:fe12:a02%3]:9000"
      ];
    };

    two = { ... }: mkNode {
      vlans = [ 20 ];
      ratmanArgs = [
        "--peers inet#[fe80::5054:ff:fe12:1402%3]:9000"
      ];
    };

    three = { ... }: mkNode {
      vlans = [ 10 20 ];
      ratmanArgs = [
        "--inet [::]:9000"
      ];
    };
  };
  
  testScript = { nodes, ... }: ''
    start_all()

    one.wait_for_unit("ratcat-recv.service")
    two.wait_for_unit("ratcat-recv.service")
    three.wait_for_unit("ratcat-recv.service")

    one_addr = one.succeed("jq -r .addr ~/.config/ratcat/config")
    two.wait_until_succeeds("ratctl --get-peers | grep {}".format(one_addr))

    two.execute("echo 'hello nixos test' | ratcat {}".format(one_addr))

    one.wait_until_succeeds("journalctl --since -1m --unit ratcat-recv --grep 'hello nixos test'")
  '';
}
