{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, ... }:
    let
      near-light-client = (crane.buildWasmContract {
        crateDirFromRoot = "near/near-light-client";
        extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
        extraNativeBuildInputs = [ pkgs.clang ];
      });
    in
    {
      inherit (near-light-client) packages;
    };
}

      # workspace = (crane.buildWasmContract {
      #   crateDirFromRoot = "light-clients/cometbls-light-client";
      #   checks = [
      #     (file_path: ''
      #       ${ensure-wasm-client-type {
      #         inherit file_path;
      #         type = "Cometbls";
      #       }}
      #     '')
      #   ];
      # });
