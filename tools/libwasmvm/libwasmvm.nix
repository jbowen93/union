{ ... }: {
  perSystem = { pkgs, self', crane, rust, system, ... }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      craneLib = crane.lib.overrideToolchain (rust.mkNightly { target = CARGO_BUILD_TARGET; });

      wasmvm = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.3.0"; # wasmd 0.41.0
        hash = "sha256-rsTYvbkYpDkUE4IvILdSL3hXMgAWxz5ltGotJB2t1e4=";
      };
    in
    {
      packages.libwasmvm =
        craneLib.buildPackage (
          {
            name = "libwasmvm";
            version = wasmvm.rev;
            src = "${wasmvm}/libwasmvm";
            doCheck = false;
            inherit CARGO_BUILD_TARGET;
          } // (if pkgs.stdenv.isLinux then {
            cargoBuildCommand = "cargo build --release --example=wasmvmstatic";
            installPhaseCommand = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          } else if pkgs.stdenv.isDarwin then {
            # non-static dylib build on macOS
            cargoBuildCommand = "cargo build --release";
            installPhaseCommand = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
            '';
          } else throwBadSystem)
        );

    };
}
