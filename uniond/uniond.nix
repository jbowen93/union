{ ... }: {
  perSystem = { pkgs, self', crane, system, ... }: {
    packages = {
      bls-eth = pkgs.pkgsStatic.stdenv.mkDerivation rec {
        pname = "bls-eth";
        version = "1.31.0";
        src = pkgs.fetchFromGitHub {
          owner = "herumi";
          repo = "bls-eth-go-binary";
          rev = "f61456b875d2649d0a9c58ed5f269c82d549672a";
          hash = "sha256-pjPTrYxoKQ7CWuscMWCE0tD/Rd0xUrP4ypuGP8yezis=";
          fetchSubmodules = true;
        };
        nativeBuildInputs = [ pkgs.pkgsStatic.nasm ];
        installPhase = ''
          mkdir -p $out/lib
          mv bls/lib/linux/amd64/*.a $out/lib
        '';
        enableParallelBuilding = true;
        doCheck = true;
      };

      # Statically link on Linux using `pkgsStatic`, dynamically link on Darwin using normal `pkgs`.
      uniond = (if pkgs.stdenv.isLinux then
        pkgs.pkgsStatic.buildGoModule
      else
        pkgs.buildGoModule) ({
        name = "uniond";
        src = ./.;
        vendorSha256 = null;
        doCheck = true;
        meta.mainProgram = "uniond";
      } // (
        let libwasmvm = self'.packages.libwasmvm;
        in if pkgs.stdenv.isLinux then {
          # Statically link if we're on linux
          nativeBuildInputs = [ pkgs.musl libwasmvm ];
          ldflags = [
            "-linkmode external"
            "-extldflags '-z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib -L${self'.packages.bls-eth}/lib'"
          ];
          CGO_CFLAGS =
            "-I${pkgs.libblst}/include -I${pkgs.libblst.src}/src -I${pkgs.libblst.src}/build -I${self'.packages.bls-eth.src}/bls/include";
        } else if pkgs.stdenv.isDarwin then {
          # Dynamically link if we're on darwin by wrapping the program
          # such that the DYLD_LIBRARY_PATH includes libwasmvm 
          buildInputs = [ pkgs.makeWrapper libwasmvm pkgs.libblst ];
          postFixup = ''
            wrapProgram $out/bin/uniond \
            --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
          '';
        } else
          { }
      ));

      uniond-image = pkgs.dockerTools.buildImage {
        name = "uniond";

        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [ pkgs.coreutils-full pkgs.cacert ];
          pathsToLink = [ "/bin" ];
        };
        config = {
          Entrypoint = [ (pkgs.lib.getExe self'.packages.uniond) ];
          Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
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

      go-staticcheck = pkgs.go.stdenv.mkDerivation {
        name = "go-staticcheck";
        buildInputs = [ pkgs.go pkgs.go-tools ];
        src = ./.;
        doCheck = true;
        checkPhase = ''
          # Go will try to create a .cache/ dir in $HOME.
          # We avoid this by setting $HOME to the builder directory
          export HOME=$(pwd)

          staticcheck ./...
          touch $out
        '';
      };

    };
  };
}
