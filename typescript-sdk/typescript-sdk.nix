{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ nodejs_20 pkg-config ];
    in
    {
      packages = {
        typescript-sdk = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-HarCdLQ4dLGjImqNSrCFXjcLfqzgfxMshf40USckY3w=";
          src = ./.;
          pname = "@unionlabs/client";
          version = "0.0.0";
          nativeBuildInputs = pkgsDeps;
          buildInputs = pkgsDeps;

          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';
          doDist = false;
        };
      };
      apps = { };
    };
}
