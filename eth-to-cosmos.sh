RUST_LOG=beacon_api=debug,relayer=debug,info cargo run -p relayer -- transfer --ics20-transfer-address 0x1516B595237ef89f69fd2e6a3BcF0EbF5834F117 --ics20-bank-address 0xE63d3C2A800b6127AaDc4d77c1ECABAC80f13a33 --amount 18446744073709551615 --denom "transfer/channel-0/stake" --receiver union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2 --wallet 4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77 --source-port transfer --source-channel channel-0 --eth-rpc-api ws://0.0.0.0:8546
