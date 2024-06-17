{ self, ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      sentinel = crane.buildWorkspaceMember {
        crateDirFromRoot = "sentinel";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = sentinel.packages;

    };
  flake.nixosModules.sentinel = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.sentinel;
    in
    {
      options.services.sentinel = {
        enable = mkEnableOption "Sentinel service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.sentinel;
        };
        db_url = mkOption {
          type = types.str;
        };
        ethereum = mkOption {
          type = types.attrs;
        };
        osmosis = mkOption {
          type = types.attrs;
        };
        union = mkOption {
          type = types.attrs;
        };
        interactions = mkOption {
          type = types.listOf types.attrs;
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to sentinel";
          example = "sentinel=info";
        };
        log-format = mkOption {
          type = types.enum [ "json" "text" ];
          default = "json";
          example = "text";
        };
      };
      config =
        let
          configJson = pkgs.writeText "config.json" (builtins.toJSON {
            ethereum = cfg.ethereum;
            osmosis = cfg.osmosis;
            union = cfg.union;
            interactions = cfg.interactions;
            db_url = cfg.db_url;
          });

        in
        mkIf cfg.enable {
          systemd.services.sentinel = {
            description = "Sentinel";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];
            serviceConfig = {
              Type = "simple";
              ExecStart = "${pkgs.lib.getExe cfg.package} --config ${configJson}";
              Restart = "always";
              RestartSec = 10;
              User = "sentinel";
              Group = "sentinel";
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
            };
          };
        };
    };
}
