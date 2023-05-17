{ pkgs, prysm-beacon-chain, config, sharedVolume, ... }:
let
  prysm-beacon-init = pkgs.writeShellApplication {
    name = "prysm-beacon-init";
    runtimeInputs = [
      pkgs.curl
      pkgs.jq
      prysm-beacon-chain
      config
    ];
    text = ''
      ${pkgs.lib.getExe prysm-beacon-chain} \
        --datadir=./beacondata \
        --min-sync-peers=0 \
        --genesis-state=/${sharedVolume}/genesis.ssz \
        --interop-eth1data-votes \
        --bootstrap-node= \
        --chain-config-file=${config}/beacon-config.yml \
        --chain-id=32382 \
        --rpc-host=0.0.0.0 \
        --grpc-gateway-host=0.0.0.0 \
        --execution-endpoint=http://geth:8551 \
        --accept-terms-of-use \
        --jwt-secret=${config}/dev-jwt.prv \
        --suggested-fee-recipient=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD
    '';
  };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [ prysm-beacon-init ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    ports = [
      # gRPC
      "3500:3500"
      # RPC
      "4000:4000"
      "8080:8080"
    ];
    volumes = [
      "${sharedVolume}:/${sharedVolume}"
    ];
    command = [ (pkgs.lib.getExe prysm-beacon-init) ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
      prysm-bootstrap = {
        condition = "service_completed_successfully";
      };
    };
  };
}
