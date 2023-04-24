{ inputs, ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {
      uniond = pkgs.buildGoModule rec {
        name = "uniond";
        src = ./.;
        vendorSha256 = null;
        doCheck = true;
      };

      uniond-image = pkgs.dockerTools.buildImage {
        name = "uniond";
        config = {
          Cmd = [ "${self'.packages.uniond}/bin/uniond" ];
        };
      };
    };

    checks = {
      go-test = pkgs.go.stdenv.mkDerivation {
        name = "go-test";
        buildInputs = [ pkgs.go ];
        src = ./.;
        doCheck = true;
        checkPhase = ''
          # Go will try to create a .cache/ dir in $HOME.
          # We avoid this by setting $HOME to the builder directory
          export HOME=$(pwd)

          go version
          go test ./...
          touch $out
        '';
      };

      go-vet = pkgs.go.stdenv.mkDerivation {
        name = "go-vet";
        buildInputs = [ pkgs.go ];
        src = ./.;
        doCheck = true;
        checkPhase = ''
          # Go will try to create a .cache/ dir in $HOME.
          # We avoid this by setting $HOME to the builder directory
          export HOME=$(pwd)

          go version
          go vet ./...
          touch $out
        '';
      };
    };
  };
}
