{ self, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      swapDotsWithUnderscores = pkgs.lib.replaceStrings [ "." ] [ "_" ];

      unionvisorAll = crane.buildWorkspaceMember {
        crateDirFromRoot = "unionvisor";
        additionalTestSrcFilter = path: _type: pkgs.lib.hasPrefix "unionvisor/src/testdata/" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
          preConfigureHooks = [
            "cp -r ${self'.packages.uniond}/bin/uniond $PWD/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis\n"
            "echo 'patching testdata'\n"
            "patchShebangs $PWD/unionvisor/src/testdata\n"
          ];
        };
      };

      mkBundle = name: versions: meta: pkgs.linkFarm "union-bundle-${name}" ([
        {
          name = "meta.json";
          path = pkgs.writeText "meta.json" (builtins.toJSON meta);
        }
        {
          name = "unionvisor";
          path = "${unionvisorAll.packages.unionvisor}/bin/unionvisor";
        }
      ] ++ map
        (version: {
          name = "${meta.versions_directory}/${version}/${meta.binary_name}";
          path = pkgs.lib.getExe inputs'."${swapDotsWithUnderscores version}".packages.uniond;
        })
        versions);
    in
    {
      inherit (unionvisorAll) checks;
      packages = {
        inherit (unionvisorAll.packages) unionvisor;

        bundle-testnet = mkBundle "testnet" [ "v0.8.0" "v0.9.0" ] {
          binary_name = "uniond";
          versions_directory = "versions";
          fallback_version = "v0.8.0";
        };
      };
    };

  flake.nixosModules.unionvisor = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.unionvisor;
    in
    {
      options.services.unionvisor = {
        enable = mkEnableOption "Unionvisor service";
        bundle = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.bundle-testnet;
        };
        moniker = mkOption {
          type = types.str;
        };
      };

      config = mkIf cfg.enable {
        systemd.services.unionvisor =
          let
            unionvisor-systemd-script = pkgs.writeShellApplication {
              name = "unionvisor-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.bundle ];
              text = ''
                ${pkgs.coreutils}/bin/mkdir -p /var/lib/unionvisor 
                cd /var/lib/unionvisor 
                HOME=/var/lib/unionvisor ${cfg.bundle}/unionvisor --root /var/lib/unionvisor init --bundle ${cfg.bundle} --moniker ${cfg.moniker} --allow-dirty 
                HOME=/var/lib/unionvisor ${cfg.bundle}/unionvisor --root /var/lib/unionvisor run --bundle ${cfg.bundle} 
              '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Unionvisor";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe unionvisor-systemd-script;
              Restart = mkForce "always";
            };
          };
      };
    };
}
