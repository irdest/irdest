# SPDX-FileCopyrightText: 2022 Yaya <github@uwu.is>
# SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore
{ pkgs ? import ../. }:

pkgs.nixosTest {
  name = "ratman-two-nodes";

  nodes =
    let
      commonArgs = {
        imports = [
          ../nixos-modules/ratman.nix
        ];

        environment.systemPackages = with pkgs; [ ratman jq ];

        networking.firewall.allowedUDPPorts = [ 5861 ];

        services.ratman = {
          enable = true;
          extraArgs = [ "-v debug" "--accept-unknown-peers" "--discovery-iface eth1" ];
        };

        systemd.services = {
          ratcat-register = {
            wantedBy = [ "multi-user.target" ];
            after = [ "ratmand.service" ];
            serviceConfig = {
              ExecStart = "${pkgs.ratman}/bin/ratcat --register";
              Type = "oneshot";
              RemainAfterExit = true;
            };
          };
          ratcat-recv = {
            wantedBy = [ "multi-user.target" ];
            after = [ "ratcat-register.service" ];
            serviceConfig = {
              ExecStart = "${pkgs.ratman}/bin/ratcat --recv";
            };
          };
        };
      };
  in {
    one = { ... }: commonArgs;

    two = { ... }: commonArgs;
  };
  
  testScript = { nodes, ... }: ''
    start_all()

    one.wait_for_unit("ratcat-recv.service")
    two.wait_for_unit("ratcat-recv.service")

    one_addr = one.succeed("jq -r .addr ~/.config/ratcat/config")

    two.wait_until_succeeds("ratctl --get-peers | grep {}".format(one_addr))

    two.succeed("echo 'hello nixos test' | ratcat {}".format(one_addr))

    one.wait_until_succeeds("journalctl --since -1m --unit ratcat-recv --grep 'hello nixos test'")
  '';
}
