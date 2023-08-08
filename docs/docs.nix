{ ... }: {
  perSystem = { pkgs, lib, ... }:
    let
      npmDepsHash = "sha256-0uH8gSnftFGVQfdoj8jxbxMhQb+/FTmrbQb+E0qfS68=";
      src = ../.;
      pname = "docs";
      version = "0.0.1";
    in
    {
      packages = {

        docs = pkgs.mkYarnPackage rec {
          inherit pname version src;
          name = pname;
          packageJSON = src + "/package.json";

          offlineCache = pkgs.fetchYarnDeps {
            yarnLock = src + "/yarn.lock";
            sha256 = npmDepsHash;
          };

          nativeBuildInputs = [
            pkgs.fixup_yarn_lock
          ];

          configurePhase = ''
            export HOME=$NIX_BUILD_TOP
            yarn config --offline set yarn-offline-mirror ${offlineCache}
            fixup_yarn_lock yarn.lock
            yarn install --offline --ignore-optional --frozen-lockfile --ignore-scripts --no-progress --non-interactive
            patchShebangs node_modules/
          '';

          postBuild = "yarn workspace docs --offline --no-progress build";

          installPhase = ''
            mkdir -p $out
            cp -r ./docs/build/* $out
          '';

          doDist = false;
        };
      };

      apps = {
        docs-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "docs-dev-server";
            runtimeInputs = [ pkgs.nodejs ];
            text = ''
              yarn install --offline
              yarn workspace docs run start
            '';
          };
        };
      };
    };
}
