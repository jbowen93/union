use std::collections::BTreeMap;

use chain_utils::private_key::PrivateKey;
use ethers::prelude::k256::ecdsa;
use hubble::hasura::HasuraConfig;
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::ethereum::Address;

use crate::chain::AnyChain;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Map of chain name to it's respective config.
    pub chain: BTreeMap<String, ChainConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoyagerConfig {
    pub hasura: Option<HasuraConfig>,
}

impl Config {
    pub async fn get_chain(&self, name: &str) -> Option<AnyChain> {
        match self.chain.get(name) {
            Some(config) => Some(AnyChain::try_from_config(&self.voyager, config.clone()).await),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "chain_type")]
pub enum ChainConfig {
    Evm(EvmChainConfig),
    Union(UnionChainConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "preset_base")]
pub enum EvmChainConfig {
    Mainnet(EvmChainConfigFields),
    Minimal(EvmChainConfigFields),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmChainConfigFields {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: Address,

    /// The signer that will be used to submit transactions by voyager.
    pub signer: PrivateKey<ecdsa::SigningKey>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionChainConfig {
    pub signer: PrivateKey<ecdsa::SigningKey>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub dump_path: String,
    pub grpc_url: String,
}

// pub mod private_key {
//     use std::fmt::{Display, Write};

//     use bip32::PrivateKeyBytes;
//     use chain_utils::private_key::PrivateKey;
//     use clap::error::{ContextKind, ContextValue};

//     #[allow(clippy::needless_pass_by_value)] // required by StringValueParser::try_map
//     pub fn parse_private_key_arg<T: bip32::PrivateKey + Clone + Send + Sync + 'static>(
//         s: String,
//     ) -> Result<PrivateKey<T>, clap::error::Error> {
//         fn invalid_value<E: Display>(s: &String, maybe_error: Option<E>) -> clap::error::Error {
//             let mut error = clap::Error::new(clap::error::ErrorKind::ValueValidation);

//             error.insert(
//                 ContextKind::InvalidValue,
//                 ContextValue::String(s.to_string()),
//             );

//             let mut usage_and_note = "Usage: raw:<private key hex>".to_string();

//             if let Some(note) = maybe_error {
//                 write!(&mut usage_and_note, "\n\nNote: {note}").unwrap();
//             }

//             error.insert(
//                 ContextKind::Usage,
//                 ContextValue::StyledStr(usage_and_note.into()),
//             );
//             error
//         }

//         match s.as_bytes() {
//             [b'r', b'a', b'w', b':', pk @ ..] => serde_utils::parse_hex::<PrivateKeyBytes>(pk)
//                 .map_err(|err| invalid_value(&s, Some(err)))
//                 .and_then(|pk: PrivateKeyBytes| {
//                     T::from_bytes(&pk).map_err(|err| invalid_value(&s, Some(err)))
//                 })
//                 .map(PrivateKey::Raw),
//             // NOTE: &str holds no meaning here, but a type is required since it's not possible to set a
//             // default for E: https://github.com/rust-lang/rust/issues/36887
//             _ => Err(invalid_value(&s, None::<&str>)),
//         }
//     }
// }
