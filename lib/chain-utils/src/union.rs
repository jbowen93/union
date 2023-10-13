use std::{fmt::Display, num::ParseIntError, str::FromStr};

use ethers::prelude::k256::ecdsa;
use futures::{stream, Future, FutureExt, Stream, StreamExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tendermint_rpc::{
    query::{Condition, EventType, Operand, Query},
    Client, Order, SubscriptionClient, WebSocketClient, WebSocketClientUrl,
};
use unionlabs::{
    ethereum::H256,
    events::{IbcEvent, TryFromTendermintEventError, WriteAcknowledgement},
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        google::protobuf::{any::Any, duration::Duration},
        lightclients::{cometbls, wasm},
    },
    id::Id,
    id_type,
    tendermint::abci::{event::Event, event_attribute::EventAttribute},
    CosmosAccountId,
};

use crate::{private_key::PrivateKey, Chain, ChainEvent, ClientState, EventSource, Pool};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub signers: Pool<CosmosAccountId>,
    pub fee_denom: String,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

impl Chain for Union {
    type SelfClientState =
        Any<wasm::client_state::ClientState<cometbls::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>>;

    type Header = cometbls::header::Header;

    type Height = Height;

    type ClientId = UnionClientId;

    type ClientType = UnionClientType;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .tm_client
                .latest_block()
                .await
                .unwrap()
                .block
                .header
                .height
                .value()
                // HACK: for some reason, abci_query on latest block return null
                // value sometimes, probably a racy condition if we use the
                // actually latest block being built?
                .saturating_sub(1);

            self.make_height(height)
        }
    }

    fn query_latest_height_as_destination(&self) -> impl Future<Output = Height> + '_ {
        self.query_latest_height()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = i64> + '_ {
        async move {
            let height = self.query_latest_height().await;
            self.tm_client
                .block(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap()
                .block
                .header
                .time
                .unix_timestamp()
        }
    }

    fn self_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
            let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
                self.grpc_url.clone(),
            )
            .await
            .unwrap()
            .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
            .await
            .unwrap()
            .into_inner()
            .params
            .unwrap();

            let commit = self
                .tm_client
                .commit(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            let height = commit.signed_header.header.height;

            let unbonding_period = std::time::Duration::new(
                params
                    .unbonding_time
                    .clone()
                    .unwrap()
                    .seconds
                    .try_into()
                    .unwrap(),
                params
                    .unbonding_time
                    .clone()
                    .unwrap()
                    .nanos
                    .try_into()
                    .unwrap(),
            );

            Any(wasm::client_state::ClientState {
                data: cometbls::client_state::ClientState {
                    chain_id: self.chain_id.clone(),
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    trusting_period: Duration::new(
                        (unbonding_period * 85 / 100).as_secs() as i64,
                        (unbonding_period * 85 / 100).subsec_nanos() as i32,
                    )
                    .unwrap(),
                    unbonding_period: Duration::new(
                        unbonding_period.as_secs() as i64,
                        unbonding_period.subsec_nanos() as i32,
                    )
                    .unwrap(),
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    max_clock_drift: Duration::new(60 * 10, 0).unwrap(),
                    frozen_height: Height {
                        revision_number: 0,
                        revision_height: 0,
                    },
                },
                // TODO: Get this somehow
                code_id: H256::default(),
                latest_height: Height {
                    revision_number: self.chain_id.split('-').last().unwrap().parse().unwrap(),
                    revision_height: height.value(),
                },
            })
        }
    }

    fn self_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            let state = cometbls::consensus_state::ConsensusState {
                root: MerkleRoot {
                    hash: commit
                        .signed_header
                        .header
                        .app_hash
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .unwrap(),
                },
                next_validators_hash: commit
                    .signed_header
                    .header
                    .next_validators_hash
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .unwrap(),
            };

            Any(wasm::consensus_state::ConsensusState {
                data: state,
                timestamp: commit
                    .signed_header
                    .header
                    .time
                    .unix_timestamp()
                    .try_into()
                    .unwrap(),
            })
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: String,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let block_height = self
                .tm_client
                .block_by_hash(block_hash.0.to_vec().try_into().unwrap())
                .await
                .unwrap()
                .block
                .unwrap()
                .header
                .height;

            let wa = self
                .tm_client
                .tx_search(
                    Query::from(EventType::Tx).and_eq("tx.height", u64::from(block_height)),
                    false,
                    1,
                    255,
                    tendermint_rpc::Order::Ascending,
                )
                .await
                .unwrap()
                .txs
                .into_iter()
                .find_map(|tx| {
                    tx.tx_result.events.into_iter().find_map(|event| {
                        let maybe_ack = WriteAcknowledgement::try_from(
                            unionlabs::tendermint::abci::event::Event {
                                ty: event.kind,
                                attributes: event.attributes.into_iter().map(|attr| {
                                    unionlabs::tendermint::abci::event_attribute::EventAttribute {
                                        key: attr.key,
                                        value: attr.value,
                                        index: attr.index,
                                    }
                                }).collect()
                            },
                        );

                        match maybe_ack {
                            Ok(ok)
                                if ok.packet_sequence == sequence
                                    && ok.packet_src_port == destination_port_id
                                    && ok.packet_src_channel == destination_channel_id =>
                            {
                                Some(ok)
                            }
                            Ok(_) => None,
                            Err(TryFromTendermintEventError::IncorrectType { .. }) => None,
                            Err(err) => {
                                panic!("{err:#?}")
                            }
                        }
                    })
                })
                .unwrap();

            wa.packet_ack_hex
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnionInitError {
    #[error("tendermint rpc error")]
    Tendermint(#[from] tendermint_rpc::Error),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    // TODO: Once the `Id` trait in unionlabs is cleaned up to no longer use static id types, this error should just wrap `IdParseError`
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

impl Union {
    pub async fn new(config: Config) -> Result<Self, UnionInitError> {
        let (tm_client, driver) = WebSocketClient::builder(config.ws_url)
            .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
            .build()
            .await?;

        tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            signers: Pool::new(
                config
                    .signers
                    .into_iter()
                    .map(|signer| CosmosAccountId::new(signer.value(), "union".to_string())),
            ),
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            grpc_url: config.grpc_url,
            fee_denom: config.fee_denom,
        })
    }

    pub async fn broadcast_tx_commit(
        &self,
        signer: CosmosAccountId,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
    ) {
        use protos::cosmos::tx;

        'construct_tx: loop {
            let account = self.account_info_of_signer(&signer).await;

            let sign_doc = tx::v1beta1::SignDoc {
                body_bytes: tx::v1beta1::TxBody {
                    messages: messages.clone().into_iter().collect(),
                    // TODO(benluelo): What do we want to use as our memo?
                    memo: String::new(),
                    timeout_height: 123_123_123,
                    extension_options: vec![],
                    non_critical_extension_options: vec![],
                }
                .encode_to_vec(),
                auth_info_bytes: tx::v1beta1::AuthInfo {
                    signer_infos: [tx::v1beta1::SignerInfo {
                        public_key: Some(protos::google::protobuf::Any {
                            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                            value: signer.public_key().encode_to_vec(),
                        }),
                        mode_info: Some(tx::v1beta1::ModeInfo {
                            sum: Some(tx::v1beta1::mode_info::Sum::Single(
                                tx::v1beta1::mode_info::Single {
                                    mode: tx::signing::v1beta1::SignMode::Direct.into(),
                                },
                            )),
                        }),
                        sequence: account.sequence,
                    }]
                    .to_vec(),
                    fee: Some(tx::v1beta1::Fee {
                        amount: vec![protos::cosmos::base::v1beta1::Coin {
                            // TODO: This needs to be configurable
                            denom: self.fee_denom.clone(),
                            amount: "1".to_string(),
                        }],
                        gas_limit: 5_000_000_000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                }
                .encode_to_vec(),
                chain_id: self.chain_id.clone(),
                account_number: account.account_number,
            };

            let alice_signature = signer.try_sign(&sign_doc.encode_to_vec()).unwrap().to_vec();

            let tx_raw = tx::v1beta1::TxRaw {
                body_bytes: sign_doc.body_bytes,
                auth_info_bytes: sign_doc.auth_info_bytes,
                signatures: [alice_signature].to_vec(),
            };

            let tx_raw_bytes = tx_raw.encode_to_vec();

            let tx_hash =
                hex::encode_upper(sha2::Sha256::new().chain_update(&tx_raw_bytes).finalize());

            let query = Query {
                event_type: Some(EventType::Tx),
                conditions: [Condition::eq(
                    "tx.hash".to_string(),
                    Operand::String(tx_hash.clone()),
                )]
                .into(),
            };

            loop {
                if self
                    .tm_client
                    .tx(tx_hash.parse().unwrap(), false)
                    .await
                    .is_ok()
                {
                    // TODO: Log an error if this is unsuccessful
                    let _ = self.tm_client.unsubscribe(query).await;
                    return;
                }

                // dbg!(maybe_tx);

                let response_result = self.tm_client.broadcast_tx_sync(tx_raw_bytes.clone()).await;

                // dbg!(&response_result);

                let response = response_result.unwrap();

                assert_eq!(tx_hash, response.hash.to_string());

                tracing::debug!(%tx_hash);

                tracing::info!(check_tx_code = ?response.code, check_tx_log = %response.log);

                if response.code.is_err() {
                    let value: TendermintResponseErrorCode = response
                        .code
                        .value()
                        .try_into()
                        .expect("unknown response code");

                    if value == TendermintResponseErrorCode::WrongSequence {
                        tracing::warn!("sequence mismatch, reconstructing tx");
                        continue 'construct_tx;
                    }

                    panic!("check_tx failed: {:?}", response)
                };

                // HACK: wait for a block to verify inclusion
                tokio::time::sleep(std::time::Duration::from_secs(7)).await;

                let tx_inclusion = self.tm_client.tx(tx_hash.parse().unwrap(), false).await;

                tracing::debug!(?tx_inclusion);

                match tx_inclusion {
                    Ok(_) => break 'construct_tx,
                    Err(_) => {
                        // TODO: we don't handle this case, either we got an error or the tx hasn't been received
                        // we need to discriminate
                        tracing::warn!("tx inclusion couldn't be retrieved after 1 block");
                        panic!()
                    }
                };
            }
        }
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn account_info_of_signer(
        &self,
        signer: &CosmosAccountId,
    ) -> protos::cosmos::auth::v1beta1::BaseAccount {
        let account = protos::cosmos::auth::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .account(protos::cosmos::auth::v1beta1::QueryAccountRequest {
            address: signer.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

        // TODO: Type in `unionlabs` for this
        assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

        protos::cosmos::auth::v1beta1::BaseAccount::decode(&*account.value).unwrap()
    }
}

// TODO: This is for all cosmos chains; rename?
crate::chain_client_id! {
    #[ty = UnionClientType]
    pub enum UnionClientId {
        #[id(ty = "08-wasm")]
        Wasm(Id<_>),
        #[id(ty = "07-tendermint")]
        Tendermint(Id<_>),
    }
}

#[derive(Debug)]
pub enum UnionEventSourceError {
    TryFromTendermintEvent(TryFromTendermintEventError),
    Subscription(tendermint_rpc::Error),
}

impl EventSource for Union {
    type Event = ChainEvent<Self>;
    type Error = UnionEventSourceError;
    // TODO: Make this the height to start from
    type Seed = ();

    fn events(self, _seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> {
        async move {
            let chain_revision = self.chain_revision;

            let latest_height = self.query_latest_height().await;

            stream::unfold(
                (self, latest_height),
                move |(this, previous_height)| async move {
                    tracing::info!("fetching events");

                    let current_height = loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let current_height = this.query_latest_height().await;

                        tracing::debug!(%current_height, %previous_height);

                        if current_height > previous_height {
                            break current_height;
                        }
                    };

                    tracing::debug!(
                        previous_height = previous_height.revision_height,
                        current_height = current_height.revision_height
                    );

                    let mut events = vec![];

                    for h in
                        (previous_height.revision_height + 1)..=(current_height.revision_height)
                    {
                        let response = this
                            .tm_client
                            .tx_search(Query::eq("tx.height", h), false, 1, 255, Order::Descending)
                            .await
                            .unwrap();

                        let new_events = stream::iter(response.txs.into_iter().flat_map(|tx| {
                            tx.tx_result
                                .events
                                .into_iter()
                                .map(|event| Event {
                                    ty: event.kind,
                                    attributes: event
                                        .attributes
                                        .into_iter()
                                        .map(|attr| EventAttribute {
                                            key: attr.key,
                                            value: attr.value,
                                            index: attr.index,
                                        })
                                        .collect(),
                                })
                                .filter_map(IbcEvent::try_from_tendermint_event)
                                .map(move |res| {
                                    res.map(|x| (tx.height, x))
                                        .map_err(UnionEventSourceError::TryFromTendermintEvent)
                                })
                        }))
                        .then(|res| async {
                            match res {
                                Ok((height, event)) => Ok(ChainEvent {
                                    chain_id: this.chain_id(),
                                    block_hash: this
                                        .tm_client
                                        .block(height)
                                        .await
                                        .unwrap()
                                        .block_id
                                        .hash
                                        .as_bytes()
                                        .try_into()
                                        .unwrap(),
                                    height: Height {
                                        revision_number: chain_revision,
                                        revision_height: height.try_into().unwrap(),
                                    },
                                    event,
                                }),
                                Err(err) => Err(err),
                            }
                        })
                        .collect::<Vec<_>>()
                        .await;

                        events.extend(new_events);
                    }

                    let iter = events;

                    Some((iter, (this, current_height)))
                },
            )
        }
        .flatten_stream()
        .map(futures::stream::iter)
        .flatten()
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, num_enum::IntoPrimitive, num_enum::TryFromPrimitive,
)]
#[repr(u32)]
#[allow(non_upper_case_globals)] // TODO: Report this upstream
pub enum TendermintResponseErrorCode {
    // errInternal should never be exposed, but we reserve this code for non-specified errors
    Internal = 1,

    // ErrTxDecode is returned if we cannot parse a transaction
    TxDecode = 2,

    // ErrInvalidSequence is used the sequence number (nonce) is incorrect
    // for the signature
    InvalidSequence = 3,

    // ErrUnauthorized is used whenever a request without sufficient
    // authorization is handled.
    Unauthorized = 4,

    // ErrInsufficientFunds is used when the account cannot pay requested amount.
    InsufficientFunds = 5,

    // ErrUnknownRequest to doc
    UnknownRequest = 6,

    // ErrInvalidAddress to doc
    InvalidAddress = 7,

    // ErrInvalidPubKey to doc
    InvalidPubKey = 8,

    // ErrUnknownAddress to doc
    UnknownAddress = 9,

    // ErrInvalidCoins to doc
    InvalidCoins = 10,

    // ErrOutOfGas to doc
    OutOfGas = 11,

    // ErrMemoTooLarge to doc
    MemoTooLarge = 12,

    // ErrInsufficientFee to doc
    InsufficientFee = 13,

    // ErrTooManySignatures to doc
    TooManySignatures = 14,

    // ErrNoSignatures to doc
    NoSignatures = 15,

    // ErrJSONMarshal defines an ABCI typed JSON marshalling error
    JSONMarshal = 16,

    // ErrJSONUnmarshal defines an ABCI typed JSON unmarshalling error
    JSONUnmarshal = 17,

    // ErrInvalidRequest defines an ABCI typed error where the request contains
    // invalid data.
    InvalidRequest = 18,

    // ErrTxInMempoolCache defines an ABCI typed error where a tx already exists
    // in the mempool.
    TxInMempoolCache = 19,

    // ErrMempoolIsFull defines an ABCI typed error where the mempool is full.
    MempoolIsFull = 20,

    // ErrTxTooLarge defines an ABCI typed error where tx is too large.
    TxTooLarge = 21,

    // ErrKeyNotFound defines an error when the key doesn't exist
    KeyNotFound = 22,

    // ErrWrongPassword defines an error when the key password is invalid.
    WrongPassword = 23,

    // ErrorInvalidSigner defines an error when the tx intended signer does not match the given signer.
    orInvalidSigner = 24,

    // ErrorInvalidGasAdjustment defines an error for an invalid gas adjustment
    orInvalidGasAdjustment = 25,

    // ErrInvalidHeight defines an error for an invalid height
    InvalidHeight = 26,

    // ErrInvalidVersion defines a general error for an invalid version
    InvalidVersion = 27,

    // ErrInvalidChainID defines an error when the chain-id is invalid.
    InvalidChainID = 28,

    // ErrInvalidType defines an error an invalid type.
    InvalidType = 29,

    // ErrTxTimeoutHeight defines an error for when a tx is rejected out due to an
    // explicitly set timeout height.
    TxTimeoutHeight = 30,

    // ErrUnknownExtensionOptions defines an error for unknown extension options.
    UnknownExtensionOptions = 31,

    // ErrWrongSequence defines an error where the account sequence defined in
    // the signer info doesn't match the account's actual sequence number.
    WrongSequence = 32,

    // ErrPackAny defines an error when packing a protobuf message to Any fails.
    PackAny = 33,

    // ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
    UnpackAny = 34,

    // ErrLogic defines an internal logic error, e.g. an invariant or assertion
    // that is violated. It is a programmer error, not a user-facing error.
    Logic = 35,

    // ErrConflict defines a conflict error, e.g. when two goroutines try to access
    // the same resource and one of them fails.
    Conflict = 36,

    // ErrNotSupported is returned when we call a branch of a code which is currently not
    // supported.
    NotSupported = 37,

    // ErrNotFound defines an error when requested entity doesn't exist in the state.
    NotFound = 38,

    // ErrIO should be used to wrap internal errors caused by external operation.
    // Examples: not DB domain error, file writing etc...
    IO = 39,

    // ErrPanic is only set when we recover from a panic, so we know to
    // redact potentially sensitive system info
    Panic = 111222,

    // ErrAppConfig defines an error occurred if min-gas-prices field in BaseConfig is empty.
    AppConfig = 40,
}
