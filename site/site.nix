{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ nodejs_20 vips pkg-config ];
    in
    {
      packages = {
        site = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-wBT98el48zHWDKhsvbtVJ4oLUP9rZx/4wtFqkupCzCw=";
          src = ./.;
          srcs = [
            ./.
            ./../evm/.
          ];
          sourceRoot = "site";
          pname = "site";
          version = "0.0.1";
          PUPPETEER_SKIP_DOWNLOAD = true;

          # nodejs = pkgs.nodejs_20;
          nativeBuildInputs = pkgsDeps;
          buildInputs = pkgsDeps;

          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';

          doDist = false;
        };
      };

      apps = {
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            runtimeInputs = pkgsDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
