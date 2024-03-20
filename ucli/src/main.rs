use std::{fs::read_to_string, sync::Arc};

use chain_utils::{cosmos_sdk::CosmosSdkChainExt, union::Union};
use clap::Parser;
use cli::Ethereum;
use contracts::{
    erc20,
    ucs01_relay::{LocalToken, UCS01Relay},
};
use ethers::{middleware::SignerMiddleware, signers::Signer, types::Address};
use unionlabs::{
    ethereum::config::{ChainSpec, Mainnet, Minimal},
    uint::U256,
};

use crate::cli::{AppArgs, Config};

mod cli;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();
    let config = read_to_string(&args.config_file_path).unwrap();
    let config = serde_json::from_str::<Config>(&config).unwrap();

    match args.command {
        cli::Command::Tx(tx) => match tx {
            cli::TxCmd::Ethereum(ethereum_tx) => {
                match ethereum_tx {
                    cli::EthereumTx::Transfer {
                        relay_address,
                        port_id,
                        channel_id,
                        receiver,
                        amount,
                        denom,
                    } => match config.ethereum {
                        cli::EthereumChainConfig::Mainnet(config) => {
                            handle_transfer::<Mainnet>(
                                Ethereum::new(config).await.unwrap(),
                                relay_address.into(),
                                port_id,
                                channel_id,
                                receiver,
                                amount,
                                denom,
                            )
                            .await
                        }
                        cli::EthereumChainConfig::Minimal(config) => {
                            handle_transfer::<Minimal>(
                                Ethereum::new(config).await.unwrap(),
                                relay_address.into(),
                                port_id,
                                channel_id,
                                receiver,
                                amount,
                                denom,
                            )
                            .await
                        }
                    },
                };
            }
        },
        cli::Command::Query(query) => match query {
            cli::QueryCmd::Ethereum(ethereum_query) => match ethereum_query {
                cli::EthereumQuery::Ucs01Balance {
                    contract_address,
                    denom,
                    address,
                    channel_id,
                    port_id,
                } => match config.ethereum {
                    cli::EthereumChainConfig::Mainnet(config) => {
                        handle_ucs_balance::<Mainnet>(
                            Ethereum::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
                            channel_id,
                            port_id,
                        )
                        .await
                    }
                    cli::EthereumChainConfig::Minimal(config) => {
                        handle_ucs_balance::<Minimal>(
                            Ethereum::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
                            channel_id,
                            port_id,
                        )
                        .await
                    }
                },
                cli::EthereumQuery::Erc20Balance {
                    contract_address,
                    address,
                } => match config.ethereum {
                    cli::EthereumChainConfig::Mainnet(config) => {
                        handle_erc_balance::<Mainnet>(
                            Ethereum::new(config).await.unwrap(),
                            contract_address.into(),
                            address.into(),
                        )
                        .await
                    }
                    cli::EthereumChainConfig::Minimal(config) => {
                        handle_erc_balance::<Minimal>(
                            Ethereum::new(config).await.unwrap(),
                            contract_address.into(),
                            address.into(),
                        )
                        .await
                    }
                },
            },
            cli::QueryCmd::Union(union_query) => match union_query {
                cli::UnionQuery::AccountInfo { address } => {
                    let info = Union::new(chain_utils::union::Config {
                        signers: config.union.signers,
                        fee_denom: config.union.fee_denom,
                        ws_url: config.union.ws_url,
                        prover_endpoint: config.union.prover_endpoint,
                        grpc_url: config.union.grpc_url,
                    })
                    .await
                    .unwrap()
                    .account_info(&address)
                    .await;
                    println!("{info:#?}");
                }
            },
        },
    }
}

async fn handle_ucs_balance<C: ChainSpec>(
    ethereum: Ethereum<C>,
    contract_address: Address,
    denom: String,
    address: Address,
    channel_id: String,
    port_id: String,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        ethereum.provider.clone(),
        ethereum.wallet.clone(),
    ));
    let relay = UCS01Relay::new(contract_address, signer_middleware.clone());

    let denom = relay
        .get_denom_address(port_id, channel_id, denom)
        .await
        .unwrap();
    println!("Corresponding ERC20 address: {}", denom);

    let erc_contract = erc20::ERC20::new(denom, signer_middleware.clone());

    let balance = erc_contract.balance_of(address).await.unwrap();
    println!("Balance is: {}", balance);
}

async fn handle_erc_balance<C: ChainSpec>(
    ethereum: Ethereum<C>,
    contract_address: Address,
    address: Address,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        ethereum.provider.clone(),
        ethereum.wallet.clone(),
    ));
    let erc_contract = erc20::ERC20::new(contract_address, signer_middleware);

    let balance = erc_contract.balance_of(address).await.unwrap();
    println!("Balance is: {}", balance);
}

async fn handle_transfer<C: ChainSpec>(
    ethereum: Ethereum<C>,
    relay_address: Address,
    port_id: String,
    channel_id: String,
    receiver: String,
    amount: u64,
    denom: String,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        ethereum.provider.clone(),
        ethereum.wallet.clone(),
    ));
    let relay = UCS01Relay::new(relay_address, signer_middleware.clone());

    let denom = relay
        .get_denom_address(port_id.clone(), channel_id.clone(), denom)
        .await
        .unwrap();
    println!("Address is: {}", denom);

    let erc_contract = erc20::ERC20::new(denom, signer_middleware.clone());

    let balance = erc_contract
        .balance_of(ethereum.wallet.address())
        .await
        .unwrap();
    println!("Balance is: {}", balance);

    erc_contract
        .approve(relay_address, (U256::MAX / U256::from(2)).into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let tx_rcp = relay
        .send(
            port_id,
            channel_id,
            hex::decode(receiver).unwrap().into(),
            [LocalToken {
                denom,
                amount: amount.into(),
            }]
            .into(),
            u64::MAX,
            u64::MAX,
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    dbg!(tx_rcp);
}
