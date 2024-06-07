{ ... }: {
  perSystem = { biome, pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config biome ];
      nodeDeps = with unstablePkgs; [ vips nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      apps = {
        pre-commit = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "pre-commit";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}

              echo "Applying nix fmt"
              nix fmt


              echo "Applying biome fmt"
              ${lib.getExe biome} format . \
                --log-level="info" \
                --log-kind="pretty" \
                --error-on-warnings \
                --diagnostic-level="info" \
                --write

              echo "Checking spelling"
              nix build .\#checks.${pkgs.system}.spellcheck -L
            '';
          };
        };
      };
    };
}
