{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        indexer = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-LIG/5rwLcYvGDKdsUgsPYn4ElsOeVdYage9VJqLAkWU=";
          src = ./.;
          sourceRoot = "indexer";
          pname = "union-transfers-indexer";
          version = "0.0.0";
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          dontNpmBuild = true;
          installPhase = ''
            mkdir -p $out
            cp -r ./* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
          PONDER_TELEMETRY_DISABLED = true;
        };
      };

      apps = {
        app-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
