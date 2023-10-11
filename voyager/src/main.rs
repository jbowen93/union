#![recursion_limit = "256"]
// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::module_name_repetitions
)]
// #![deny(clippy::unwrap_used)]

use std::{ffi::OsString, fs::read_to_string, process::ExitCode};

use chain_utils::{evm::Evm, union::Union};
use clap::Parser;
use unionlabs::ethereum_consts_traits::Mainnet;

use crate::{
    chain::AnyChain,
    cli::{AppArgs, Command, QueryCmd},
    config::{Config, GetChainError},
    queue::{AnyQueue, Voyager, VoyagerInitError},
};

pub const DELAY_PERIOD: u64 = 0;

pub mod cli;
pub mod config;

pub mod msg;

pub mod queue;

pub mod chain;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();

    match do_main(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerError {
    #[error("unable to read the config file at {}", path.to_string_lossy())]
    ConfigFileNotFound {
        path: OsString,
        #[source]
        source: std::io::Error,
    },
    #[error("unable to parse the config file at {}", path.to_string_lossy())]
    ConfigFileParse {
        path: OsString,
        #[source]
        source: serde_json::Error,
    },
    #[error("error retrieving a chain from the config")]
    GetChain(#[from] GetChainError),
    #[error("error initializing voyager")]
    Init(#[from] VoyagerInitError<AnyQueue>),
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(args: cli::AppArgs) -> Result<(), VoyagerError> {
    let voyager_config = read_to_string(&args.config_file_path)
        .map_err(|err| VoyagerError::ConfigFileNotFound {
            path: args.config_file_path.clone(),
            source: err,
        })
        .and_then(|s| {
            serde_json::from_str::<Config<AnyQueue>>(&s).map_err(|err| {
                VoyagerError::ConfigFileParse {
                    path: args.config_file_path,
                    source: err,
                }
            })
        })?;

    match args.command {
        Command::PrintConfig => {
            println!(
                "{}",
                serde_json::to_string_pretty(&voyager_config)
                    .expect("config serialization is infallible; qed;")
            );
        }
        Command::Relay => {
            let queue = Voyager::new(voyager_config.clone()).await?;

            queue.run().await;
        }
        Command::Setup(cmd) => match cmd {
            // TODO(aeryz): this might go into channel as well, since it's highly coupled with it
            cli::SetupCmd::BindPort {
                on,
                module_address,
                port_id,
            } => {
                let chain = voyager_config.get_chain(&on).await?;

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        chain::evm::bind_port(&evm, module_address.into(), port_id).await
                    }
                    AnyChain::EvmMainnet(evm) => {
                        chain::evm::bind_port(&evm, module_address.into(), port_id).await
                    }
                    _ => panic!("Not supported"),
                };
            }
            cli::SetupCmd::InitialChannel {
                on,
                counterparty_port_id,
                module_address,
                port_id,
                channel_id,
            } => {
                let chain = voyager_config.get_chain(&on).await?;

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        chain::evm::setup_initial_channel(
                            &evm,
                            module_address.into(),
                            channel_id,
                            port_id,
                            counterparty_port_id,
                        )
                        .await;
                    }
                    _ => panic!("Not supported."),
                }
            }
            _ => panic!("not supported"),
        },
        Command::Query { on, at, cmd } => {
            let on = voyager_config.get_chain(&on).await?;

            match cmd {
                QueryCmd::IbcPath(path) => {
                    let json = match on {
                        AnyChain::EvmMainnet(evm) => {
                            path.any_state_proof_to_json::<Union, _>(evm, at).await
                        }
                        AnyChain::EvmMinimal(evm) => {
                            path.any_state_proof_to_json::<Union, _>(evm, at).await
                        }
                        AnyChain::Union(union) => {
                            // NOTE: ChainSpec is arbitrary
                            path.any_state_proof_to_json::<Evm<Mainnet>, _>(union, at)
                                .await
                        }
                    };

                    println!("{json}");
                }
            }
        }
    }

    Ok(())
}
