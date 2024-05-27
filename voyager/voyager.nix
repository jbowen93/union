{ self, ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      voyager = crane.buildWorkspaceMember {
        crateDirFromRoot = "voyager";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = voyager.packages // {
        voy-send-msg = pkgs.writeShellApplication {
          name = "voy-send-msg";
          runtimeInputs = [ pkgs.curl ];
          text = ''
            set -e
            curl localhost:65534/msg -H "content-type: application/json" -d "$@"
          '';
        };
      };
      checks = voyager.checks;
    };

  flake.nixosModules.voyager = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.voyager;
    in
    {
      options.services.voyager = {
        enable = mkEnableOption "Voyager service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.voyager;
        };
        chains = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.attrs;
        };
        workers = mkOption {
          type = types.int;
          default = 20;
        };
        runtime-max-secs = mkOption {
          type = types.int;
          default = 1800;
        };
        db-url = mkOption {
          type = types.str;
          default = "postgres://voyager:voyager@localhost/voyager";
        };
        db-min-conn = mkOption {
          type = types.int;
          default = 20;
        };
        db-max-conn = mkOption {
          type = types.int;
          default = 20;
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to voyager";
          example = "voyager=debug";
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
            chain = cfg.chains;
            voyager = {
              num_workers = cfg.workers;
              queue = {
                type = "pg-queue";
                database_url = cfg.db-url;
                min_connections = cfg.db-min-conn;
                max_connections = cfg.db-max-conn;
                idle_timeout = null;
                max_lifetime = null;
              };
            };
          });
        in
        mkIf cfg.enable {
          systemd.services.voyager-migration = {
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];
            description = "Voyager Migration";
            serviceConfig = {
              Type = "oneshot";
              ExecStart = ''
                ${pkgs.lib.meta.getExe cfg.package} \
                  --config-file-path ${configJson} \
                  -l ${cfg.log-format} \
                  run-migrations
              '';
            };
            environment = {
              RUST_LOG = "debug";
              RUST_BACKTRACE = "full";
            };
          };
          systemd.services.voyager = {
            wantedBy = [ "multi-user.target" ];
            after = [ "voyager-migration.service" ];
            partOf = [ "voyager-migration.service" ];
            requires = [ "voyager-migration.service" ];
            description = "Voyager";
            serviceConfig = {
              Type = "simple";
              ExecStart = ''
                ${pkgs.lib.getExe cfg.package} \
                  --config-file-path ${configJson} \
                  -l ${cfg.log-format} \
                  relay
              '';
              Restart = mkForce "always";
              RestartSec = 10;
              RuntimeMaxSec = cfg.runtime-max-secs;
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
            };
          };
        };
    };
}
