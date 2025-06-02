{
  config,
  lib,
  pkgs,
  ...
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../../Cargo.toml);
  inherit (cargoToml.package) description;
  cfg = config.services.wol-libvirt-gateway;
in {
  options.services.wol-libvirt-gateway = with lib; {
    enable = mkEnableOption description;

    package = mkOption {
      type = types.package;
      default = pkgs.wol-libvirt-gateway;
      defaultText = literalExpression "pkgs.wol-libvirt-gateway";
      description = "Package containing the ${description}";
    };

    address = mkOption {
      type = types.str;
      default = "127.0.0.1:9";
      description = "Listen address and port for WOL packets";
    };

    libvirtUri = mkOption {
      type = types.str;
      default = "qemu:///system";
      description = "Libvirt connection URI";
    };

    allowedSubnets = mkOption {
      type = types.listOf types.str;
      default = ["127.0.0.1/8"];
      description = "Allowed source IP subnets for WOL packets";
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.wol-libvirt-gateway = {
      isSystemUser = true;
      group = "wol-libvirt-gateway";
      extraGroups = ["libvirtd"];
      description = "WOL Libvirt Gateway service user";
    };

    users.groups.wol-libvirt-gateway = {};

    systemd.services.wol-libvirt-gateway = {
      inherit description;
      documentation = [cargoToml.package.repository];

      after = ["network.target" "libvirtd.service"];
      wants = ["libvirtd.service"];
      bindsTo = ["libvirtd.service"];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/wol-libvirt-gateway --address ${cfg.address} --libvirt-uri ${cfg.libvirtUri}";
        Restart = "always";
        RestartSec = "5s";

        ReadWritePaths = ["/var/run/libvirt/libvirt-sock" "/var/run/libvirt/libvirt-sock-ro"];
        CapabilityBoundingSet = ["CAP_NET_BIND_SERVICE"];
        AmbientCapabilities = ["CAP_NET_BIND_SERVICE"];

        # Security hardening
        User = "wol-libvirt-gateway";
        Group = "wol-libvirt-gateway";
        ProtectSystem = "strict";
        ProtectControlGroups = true;
        RemoveIPC = true;
        RestrictNamespaces = true;
        ProtectProc = "invisible";
        ProcSubset = "pid";
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectHome = true;
        NoNewPrivileges = true;
        LockPersonality = true;
        RestrictRealtime = true;
        MemoryDenyWriteExecute = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = ["@system-service"];
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;

        # Network restrictions
        IPAddressDeny = "any";
        IPAddressAllow = cfg.allowedSubnets;

        # Resource limits
        LimitNOFILE = 1024;
        LimitNPROC = 64;

        # Environment
        Environment = "RUST_LOG=info";
      };
    };
  };
}
