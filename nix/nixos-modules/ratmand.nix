# SPDX-FileCopyrightText: 2022 Yaya <github@uwu.is>
# SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

{ config, pkgs, lib, ... }:

with lib;

let
  cfg = config.services.ratmand;
in {
  options.services.ratmand = {
    enable = mkEnableOption ''
      Enable ratmand, a decentralised peer-to-peer packet router.
    '';

    extraArgs = mkOption {
      type = with types; listOf str;
      default = [];
      description = ''
        List of additional command line arguments to pass to the ratmand daemon.
      '';
    };

    package = mkOption {
      type = types.package;
      default = pkgs.ratmand;
      defaultText = literalExpression "pkgs.ratmand";
      description = ''
        Overridable attribute of the ratmand package to use.
      '';
    };
  };

  config = mkIf cfg.enable {
    #users.users.ratmand = {
    #  isSystemUser = true;
    #  group = "ratmand";
    #};

    #users.groups.ratmand = {};

    systemd.services.ratmand = {
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        #User = "ratmand";
        #Group = "ratmand";

        ExecStart = "${cfg.package}/bin/ratmand --daemonize ${builtins.concatStringsSep " " cfg.extraArgs}";
        Type = "forking";

        # Security Hardening
        # Refer to systemd.exec(5) for option descriptions.
        #CapabilityBoundingSet = "";

        # implies RemoveIPC=, PrivateTmp=, NoNewPrivileges=, RestrictSUIDSGID=,
        # ProtectSystem=strict, ProtectHome=read-only
        #DynamicUser = true;
        #LockPersonality = true;
        #PrivateDevices = true;
        #PrivateUsers = true;
        #ProcSubset = "pid";
        #ProtectClock = true;
        #ProtectControlGroups = true;
        #ProtectHome = true;
        #ProtectHostname = true;
        #ProtectKernelLogs = true;
        #ProtectProc = "invisible";
        #ProtectKernelModules = true;
        #ProtectKernelTunables = true;
        #RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        #RestrictNamespaces = true;
        #RestrictRealtime = true;
        #SystemCallArchitectures = "native";
        #SystemCallFilter = "~@clock @cpu-emulation @debug @mount @obsolete @reboot @swap @privileged @resources";
        #UMask = "0077";
      };
    };
  };
}
