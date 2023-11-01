use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use beacon_api::errors::{InternalServerError, NotFoundError};
use chain_utils::{
    evm::{CometblsMiddleware, EthCallExt, Evm, TupleToOption},
    MaybeRecoverableError,
};
use clap::Args;
use contracts::{
    ibc_handler::{
        self, AcknowledgePacketCall, ChannelOpenAckCall, ChannelOpenConfirmCall,
        ChannelOpenInitCall, ChannelOpenTryCall, ConnectionOpenAckCall, ConnectionOpenConfirmCall,
        ConnectionOpenInitCall, ConnectionOpenTryCall, CreateClientCall, GetChannelCall,
        GetClientStateCall, GetConnectionCall, GetConsensusStateCall,
        GetHashedPacketAcknowledgementCommitmentCall, GetHashedPacketCommitmentCall, IBCHandler,
        RecvPacketCall, UpdateClientCall,
    },
    shared_types::{IbcCoreChannelV1ChannelData, IbcCoreConnectionV1ConnectionEndData},
};
use ethers::{
    abi::AbiEncode,
    contract::{ContractError, EthCall},
    providers::{Middleware, ProviderError},
    types::{EIP1186ProofResponse, U256},
    utils::keccak256,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::Future;
use prost::Message;
use protos::union::ibc::lightclients::ethereum::v1 as ethereum_v1;
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{
        beacon::{GenesisData, LightClientBootstrap, LightClientFinalityUpdate},
        Address,
    },
    ethereum_consts_traits::{ChainSpec, Mainnet, Minimal},
    ibc::{
        core::{
            client::{
                height::{Height, IsHeight},
                msg_update_client::MsgUpdateClient,
            },
            connection::connection_end::ConnectionEnd,
        },
        google::protobuf::any::Any,
        lightclients::{
            ethereum::{
                self,
                account_update::AccountUpdate,
                light_client_update::LightClientUpdate,
                proof::Proof,
                trusted_sync_committee::{ActiveSyncCommittee, TrustedSyncCommittee},
            },
            wasm,
        },
    },
    id::ClientId,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::{Chain, ClientState},
    EthAbi, IntoEthAbi, IntoProto, Proto, TryFromProto, TryFromProtoErrorOf,
};

use crate::{
    chain::{
        try_from_relayer_msg,
        union::{EthereumMainnet, EthereumMinimal},
        ChainOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf, IbcStateRead, LightClient,
        LightClientBase, QueryHeight,
    },
    msg::{
        aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
        data,
        data::{
            AcknowledgementProof, ChannelEndProof, ClientConsensusStateProof, ClientStateProof,
            CommitmentProof, ConnectionProof, Data, LightClientSpecificData,
        },
        fetch,
        fetch::{
            Fetch, FetchStateProof, FetchTrustedClientState, FetchUpdateHeaders,
            LightClientSpecificFetch,
        },
        identified,
        msg::{Msg, MsgUpdateClientData},
        seq, wait,
        wait::WaitForTimestamp,
        AggregateData, AggregateReceiver, AnyLcMsg, AnyLightClientIdentified, DoAggregate,
        Identified, LcMsg, RelayerMsg,
    },
    queue::aggregate_data::{do_aggregate, UseAggregate},
};

pub const EVM_REVISION_NUMBER: u64 = 0;

/// The solidity light client, tracking the state of the 08-wasm light client on union.
pub struct CometblsMinimal {
    chain: Evm<Minimal>,
}

/// The solidity light client, tracking the state of the 08-wasm light client on union.
pub struct CometblsMainnet {
    chain: Evm<Mainnet>,
}

fn encode_dynamic_singleton_tuple(t: impl AbiEncode) -> Vec<u8> {
    U256::from(32)
        .encode()
        .into_iter()
        .chain(t.encode())
        .collect::<Vec<_>>()
}

pub async fn bind_port<C: ChainSpec>(this: &Evm<C>, module_address: Address, port_id: String) {
    // HACK: This will pop the top item out of the queue, but binding the port requires the contract owner;
    // this will work as long as the first signer in the list is the owner.
    this.ibc_handlers
        .with(|ibc_handler| async move {
            let bind_port_result = ibc_handler.bind_port(port_id, module_address.into());

            match bind_port_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => eprintln!("{:?}", why.decode_revert::<String>()),
            };
        })
        .await
}

#[allow(unused_variables)]
pub async fn setup_initial_channel<C: ChainSpec>(
    this: &Evm<C>,
    module_address: Address,
    channel_id: String,
    port_id: String,
    counterparty_port_id: String,
) {
    // let signer_middleware = Arc::new(SignerMiddleware::new(
    //     this.provider.clone(),
    //     this.wallet.clone(),
    // ));

    // let ibc_handler = devnet_ownable_ibc_handler::DevnetOwnableIBCHandler::new(
    //     this.ibc_handler.address(),
    //     signer_middleware,
    // );

    // ibc_handler
    //     .setup_initial_channel(
    //         "connection-0".into(),
    //         IbcCoreConnectionV1ConnectionEndData {
    //             client_id: "cometbls-new-0".into(),
    //             versions: vec![IbcCoreConnectionV1VersionData {
    //                 identifier: "1".into(),
    //                 features: vec!["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()],
    //             }],
    //             state: 3,
    //             counterparty: IbcCoreConnectionV1CounterpartyData {
    //                 client_id: "08-wasm-0".into(),
    //                 connection_id: "connection-0".into(),
    //                 prefix: IbcCoreCommitmentV1MerklePrefixData {
    //                     key_prefix: b"ibc".to_vec().into(),
    //                 },
    //             },
    //             delay_period: 6,
    //         },
    //         port_id,
    //         channel_id.clone(),
    //         IbcCoreChannelV1ChannelData {
    //             state: 3,
    //             ordering: 1,
    //             counterparty: IbcCoreChannelV1CounterpartyData {
    //                 port_id: counterparty_port_id,
    //                 channel_id,
    //             },
    //             connection_hops: vec!["connection-0".into()],
    //             version: "ics20-1".into(),
    //         },
    //         module_address.into(),
    //     )
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .unwrap();
    todo!()
}

impl LightClientBase for CometblsMainnet {
    type HostChain = Evm<Mainnet>;
    type Counterparty = EthereumMainnet;

    type ClientId = ClientId;
    type ClientType = String;

    type Config = CometblsConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ChannelEndPath {
                port_id,
                channel_id,
            },
            at.revision_height(),
        )
    }

    fn connection(
        &self,
        connection_id: unionlabs::id::ConnectionId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<
        Output = ConnectionEnd<
            Self::ClientId,
            <Self::Counterparty as LightClientBase>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at.revision_height(),
        )
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state(&self.chain, client_id, height)
    }
}

impl LightClient for CometblsMainnet {
    type BaseCounterparty = Self::Counterparty;

    type Data = CometblsDataMsg<Mainnet>;
    type Fetch = CometblsFetchMsg<Self, Mainnet>;
    type Aggregate = CometblsAggregateMsg<Self, Mainnet>;

    type MsgError = TxSubmitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        fetch(
            self.chain.chain_id(),
            LightClientSpecificFetch::<Self>(CometblsFetchMsg::FetchGetProof(GetProof {
                path: msg.path,
                height: msg.at,
            })),
        )
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        self::msg(&self.chain, msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(&self.chain, msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(&self.chain, update_info)
    }
}

impl LightClientBase for CometblsMinimal {
    type HostChain = Evm<Minimal>;
    type Counterparty = EthereumMinimal;

    type ClientId = ClientId;
    type ClientType = String;

    type Config = CometblsConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ChannelEndPath {
                port_id,
                channel_id,
            },
            at.revision_height(),
        )
    }

    fn connection(
        &self,
        connection_id: unionlabs::id::ConnectionId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<
        Output = ConnectionEnd<
            Self::ClientId,
            <Self::Counterparty as LightClientBase>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at.revision_height(),
        )
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state(&self.chain, client_id, height)
    }
}

impl LightClient for CometblsMinimal {
    type BaseCounterparty = Self::Counterparty;

    type Data = CometblsDataMsg<Minimal>;
    type Fetch = CometblsFetchMsg<Self, Minimal>;
    type Aggregate = CometblsAggregateMsg<Self, Minimal>;

    type MsgError = TxSubmitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        fetch(
            self.chain.chain_id(),
            LightClientSpecificFetch::<Self>(CometblsFetchMsg::FetchGetProof(GetProof {
                path: msg.path,
                height: msg.at,
            })),
        )
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        self::msg(&self.chain, msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(&self.chain, msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(&self.chain, update_info)
    }
}

async fn read_ibc_state<Counterparty, C, P>(
    evm: &Evm<C>,
    p: P,
    at: u64,
) -> P::Output
where
    Counterparty: Chain,
    C: ChainSpec,
    P: IbcPath<Evm<C>, Counterparty>
        + EthereumStateRead<
            C,
            Counterparty,
            Encoded = <<<P as EthereumStateRead<C, Counterparty>>::EthCall as EthCallExt>::Return as TupleToOption>::Inner,
        > + 'static,
    <P::EthCall as EthCallExt>::Return: TupleToOption,
{
    evm.read_ibc_state(p.into_eth_call(), at)
        .await
        .unwrap()
        .map(|x| P::decode_ibc_state(x))
        .unwrap()
}

fn generate_counterparty_updates<C, L>(
    evm: &Evm<C>,
    update_info: FetchUpdateHeaders<L>,
) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Data = CometblsDataMsg<C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    LightClientSpecificFetch<L>: From<CometblsFetchMsg<L, C>>,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    [RelayerMsg::Aggregate {
        queue: [seq([fetch(
            evm.chain_id,
            Fetch::LightClientSpecific(LightClientSpecificFetch(
                CometblsFetchMsg::FetchFinalityUpdate(PhantomData),
            )),
        )])]
        .into(),
        data: [].into(),
        receiver: AggregateReceiver::from(Identified {
            chain_id: evm.chain_id,
            data: Aggregate::LightClientSpecific(LightClientSpecificAggregate(
                CometblsAggregateMsg::MakeCreateUpdates(MakeCreateUpdatesData { req: update_info }),
            )),
        }),
    }]
    .into()
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct CreateUpdateData<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    pub req: FetchUpdateHeaders<L>,
    pub currently_trusted_slot: u64,
    pub light_client_update: LightClientUpdate<C>,
    pub is_next: bool,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeCreateUpdatesData<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    pub req: FetchUpdateHeaders<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeCreateUpdatesFromLightClientUpdatesData<
    L: LightClient<HostChain = Evm<C>>,
    C: ChainSpec,
> {
    pub req: FetchUpdateHeaders<L>,
    pub trusted_height: Height,
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchLightClientUpdate<C: ChainSpec> {
    pub period: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchLightClientUpdates<C: ChainSpec> {
    pub trusted_period: u64,
    pub target_period: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchBootstrap<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchAccountUpdate<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchBeaconGenesis<C: ChainSpec> {
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BootstrapData<C: ChainSpec> {
    pub slot: u64,
    pub bootstrap: LightClientBootstrap<C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AccountUpdateData<C: ChainSpec> {
    pub slot: u64,
    pub ibc_handler_address: Address,
    pub update: EIP1186ProofResponse,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BeaconGenesisData<C: ChainSpec> {
    genesis: GenesisData,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

try_from_relayer_msg! {
    #[CometblsMinimal(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = CometblsDataMsg,
            variants(
                FinalityUpdate(FinalityUpdate<Minimal>),
                LightClientUpdates(LightClientUpdates<Minimal>),
                LightClientUpdate(LightClientUpdate<Minimal>),
                Bootstrap(BootstrapData<Minimal>),
                AccountUpdate(AccountUpdateData<Minimal>),
                BeaconGenesis(BeaconGenesisData<Minimal>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = CometblsFetchMsg,
            variants(
                FetchFinalityUpdate(PhantomData<Minimal>),
                FetchLightClientUpdates(FetchLightClientUpdates<Minimal>),
                FetchLightClientUpdate(FetchLightClientUpdate<Minimal>),
                FetchBootstrap(FetchBootstrap<Minimal>),
                FetchAccountUpdate(FetchAccountUpdate<Minimal>),
            ),
        ),
        lc_msg(
            msg = Aggregate(LightClientSpecificAggregate),
            ty = CometblsAggregateMsg,
            variants(
                CreateUpdate(CreateUpdateData<CometblsMinimal, Minimal>),
                MakeCreateUpdates(MakeCreateUpdatesData<CometblsMinimal, Minimal>),
                MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<CometblsMinimal, Minimal>),
            ),
        ),
    )]
}

try_from_relayer_msg! {
    #[CometblsMainnet(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = CometblsDataMsg,
            variants(
                FinalityUpdate(FinalityUpdate<Mainnet>),
                LightClientUpdates(LightClientUpdates<Mainnet>),
                LightClientUpdate(LightClientUpdate<Mainnet>),
                Bootstrap(BootstrapData<Mainnet>),
                AccountUpdate(AccountUpdateData<Mainnet>),
                BeaconGenesis(BeaconGenesisData<Mainnet>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = CometblsFetchMsg,
            variants(
                FetchFinalityUpdate(PhantomData<Mainnet>),
                FetchLightClientUpdates(FetchLightClientUpdates<Mainnet>),
                FetchLightClientUpdate(FetchLightClientUpdate<Mainnet>),
                FetchBootstrap(FetchBootstrap<Mainnet>),
                FetchAccountUpdate(FetchAccountUpdate<Mainnet>),
                FetchBeaconGenesis(FetchBeaconGenesis<Mainnet>),
            ),
        ),
        lc_msg(
            msg = Aggregate(LightClientSpecificAggregate),
            ty = CometblsAggregateMsg,
            variants(
                CreateUpdate(CreateUpdateData<CometblsMainnet, Mainnet>),
                MakeCreateUpdates(MakeCreateUpdatesData<CometblsMainnet, Mainnet>),
                MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<CometblsMainnet, Mainnet>),
            ),
        ),
    )]
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum CometblsFetchMsg<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    #[display(fmt = "FetchFinalityUpdate")]
    FetchFinalityUpdate(PhantomData<C>),
    #[display(fmt = "FetchLightClientUpdates")]
    FetchLightClientUpdates(FetchLightClientUpdates<C>),
    #[display(fmt = "FetchLightClientUpdate")]
    FetchLightClientUpdate(FetchLightClientUpdate<C>),
    #[display(fmt = "FetchBootstrap")]
    FetchBootstrap(FetchBootstrap<C>),
    #[display(fmt = "FetchAccountUpdate")]
    FetchAccountUpdate(FetchAccountUpdate<C>),
    #[display(fmt = "FetchBeaconGenesis")]
    FetchBeaconGenesis(FetchBeaconGenesis<C>),
    #[display(fmt = "FetchGetProof::{}", "_0.path")]
    FetchGetProof(GetProof<C, L>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum CometblsDataMsg<C: ChainSpec> {
    #[display(fmt = "FinalityUpdate")]
    FinalityUpdate(FinalityUpdate<C>),
    #[display(fmt = "LightClientUpdates")]
    LightClientUpdates(LightClientUpdates<C>),
    #[display(fmt = "LightClientUpdate")]
    LightClientUpdate(LightClientUpdate<C>),
    #[display(fmt = "Bootstrap")]
    Bootstrap(BootstrapData<C>),
    #[display(fmt = "AccountUpdate")]
    AccountUpdate(AccountUpdateData<C>),
    #[display(fmt = "BeaconGenesis")]
    BeaconGenesis(BeaconGenesisData<C>),
}

impl<C, L> From<FinalityUpdate<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: FinalityUpdate<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::FinalityUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for FinalityUpdate<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::FinalityUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<LightClientUpdates<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: LightClientUpdates<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(
            CometblsDataMsg::LightClientUpdates(value),
        ))
    }
}

impl<C, L> From<LightClientUpdate<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: LightClientUpdate<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::LightClientUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for LightClientUpdates<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::LightClientUpdates(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> TryFrom<Data<L>> for LightClientUpdate<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::LightClientUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<BootstrapData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: BootstrapData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::Bootstrap(value)))
    }
}

impl<C, L> TryFrom<Data<L>> for BootstrapData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::Bootstrap(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<AccountUpdateData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: AccountUpdateData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::AccountUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for AccountUpdateData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::AccountUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<BeaconGenesisData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: BeaconGenesisData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::BeaconGenesis(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for BeaconGenesisData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::BeaconGenesis(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum CometblsAggregateMsg<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    #[display(fmt = "CreateUpdate")]
    CreateUpdate(CreateUpdateData<L, C>),
    #[display(fmt = "MakeCreateUpdates")]
    MakeCreateUpdates(MakeCreateUpdatesData<L, C>),
    #[display(fmt = "MakeCreateUpdatesFromLightClientUpdates")]
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<L, C>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FinalityUpdate<C: ChainSpec>(pub LightClientFinalityUpdate<C>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientUpdates<C: ChainSpec>(pub Vec<LightClientUpdate<C>>);

impl<C, L> DoAggregate<L> for CometblsAggregateMsg<L, C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Aggregate = Self, Fetch = CometblsFetchMsg<L, C>>,

    Identified<L, AccountUpdateData<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, BootstrapData<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, BeaconGenesisData<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, FinalityUpdate<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, LightClientUpdates<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, LightClientUpdate<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,

    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L::Counterparty>)>,

    AggregateData: From<identified!(Data<L>)>,
    AggregateReceiver: From<identified!(Aggregate<L>)>,
{
    fn do_aggregate(
        Identified { chain_id, data }: Identified<L, Self>,
        aggregated_data: VecDeque<AggregateData>,
    ) -> Vec<RelayerMsg> {
        [match data {
            CometblsAggregateMsg::CreateUpdate(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
            CometblsAggregateMsg::MakeCreateUpdates(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
            CometblsAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
        }]
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Args)]
pub struct CometblsConfig {
    #[arg(long)]
    pub client_type: String,
    #[arg(long)]
    pub cometbls_client_address: Address,
}

// async fn create_update(
//     &self,
//     currently_trusted_slot: u64,
//     light_client_update: LightClientUpdate<C>,
//     is_next: bool,
// ) -> wasm::header::Header<ethereum::header::Header<C>> {
//     tracing::debug!(
//         light_client_update = %serde_json::to_string(&light_client_update).unwrap(),
//         "applying light client update",
//     );

//     let bootstrap = {
//         let currently_trusted_block = self
//             .chain
//             .beacon_api_client
//             .header(beacon_api::client::BlockId::Slot(currently_trusted_slot))
//             .await
//             .unwrap()
//             .data;

//         // bootstrap contains the current sync committee for the given height
//         self.chain
//             .beacon_api_client
//             .bootstrap(currently_trusted_block.root.clone())
//             .await
//             .unwrap()
//             .data
//     };

//     let account_update_proof_height =
//         light_client_update.attested_header.execution.block_number;

//     let account_update = self
//         .chain
//         .provider
//         .get_proof(
//             self.chain.ibc_handler.address(),
//             vec![],
//             // Proofs are from the execution layer, so we use execution height, not beacon slot.
//             Some(account_update_proof_height.into()),
//         )
//         .await
//         .unwrap();

//     let header = wasm::header::Header {
//         height: self.chain.make_height(account_update_proof_height),
//         data: ethereum::header::Header {
//             consensus_update: light_client_update,
//             trusted_sync_committee: TrustedSyncCommittee {
//                 trusted_height: self
//                     .chain
//                     .make_height(bootstrap.header.execution.block_number),
//                 sync_committee: bootstrap.current_sync_committee,
//                 is_next,
//             },
//             account_update: AccountUpdate {
//                 proofs: [Proof {
//                     key: self.chain.ibc_handler.address().as_bytes().to_vec(),
//                     value: account_update.storage_hash.as_bytes().to_vec(),
//                     proof: account_update
//                         .account_proof
//                         .into_iter()
//                         .map(|x| x.to_vec())
//                         .collect(),
//                 }]
//                 .to_vec(),
//             },
//             timestamp: bootstrap.header.execution.timestamp,
//         },
//     };

//     // let new_trusted_slot = header.data.consensus_update.attested_header.beacon.slot;

//     // tracing::debug!(
//     //     "updating trusted_slot from {currently_trusted_slot} to {new_trusted_slot}"
//     // );

//     // tracing::debug!(header = %serde_json::to_string(&header).unwrap());

//     header
// }

fn make_create_update<C, L>(
    req: FetchUpdateHeaders<L>,
    chain_id: <<Evm<C> as Chain>::SelfClientState as ClientState>::ChainId,
    currently_trusted_slot: u64,
    light_client_update: LightClientUpdate<C>,
    is_next: bool,
) -> RelayerMsg
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AggregateReceiver: From<Identified<L, Aggregate<L>>>,
{
    // When we fetch the update at this height, the `next_sync_committee` will
    // be the current sync committee of the period that we want to update to.
    let previous_period = light_client_update.attested_header.beacon.slot
        / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64)
        - 1;
    RelayerMsg::Aggregate {
        queue: [
            fetch::<L>(
                chain_id,
                Fetch::LightClientSpecific(LightClientSpecificFetch(
                    CometblsFetchMsg::FetchLightClientUpdate(FetchLightClientUpdate {
                        period: previous_period,
                        __marker: PhantomData,
                    }),
                )),
            ),
            fetch::<L>(
                chain_id,
                Fetch::LightClientSpecific(LightClientSpecificFetch(
                    CometblsFetchMsg::FetchAccountUpdate(FetchAccountUpdate {
                        slot: light_client_update.attested_header.beacon.slot,
                        __marker: PhantomData,
                    }),
                )),
            ),
            fetch::<L>(
                chain_id,
                Fetch::LightClientSpecific(LightClientSpecificFetch(
                    CometblsFetchMsg::FetchBeaconGenesis(FetchBeaconGenesis {
                        __marker: PhantomData,
                    }),
                )),
            ),
        ]
        .into(),
        data: [].into(),
        receiver: AggregateReceiver::from(Identified {
            chain_id,
            data: Aggregate::LightClientSpecific(LightClientSpecificAggregate(
                CometblsAggregateMsg::CreateUpdate(CreateUpdateData {
                    req,
                    currently_trusted_slot,
                    light_client_update,
                    is_next,
                }),
            )),
        }),
    }
}

fn sync_committee_period<H: Into<u64>, C: ChainSpec>(height: H) -> u64 {
    height.into().div(C::PERIOD::U64)
}

async fn msg<C, L>(evm: &Evm<C>, msg: Msg<L>) -> Result<(), TxSubmitError>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Config = CometblsConfig>,
    ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>: Proto + IntoProto,
    ConsensusStateOf<<L::Counterparty as LightClientBase>::HostChain>: Proto + IntoProto,
    HeaderOf<<L::Counterparty as LightClientBase>::HostChain>: EthAbi + IntoEthAbi,
    // not sure why these bounds are required
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::SelfClientState: Proto,
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::SelfConsensusState: Proto,
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::Header: EthAbi,
{
    evm.ibc_handlers
        .with(|ibc_handler| async move {
            let msg: ethers::contract::FunctionCall<_, _, ()> = match msg {
                Msg::ConnectionOpenInit(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenInitCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenTry(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenTryCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenAck(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenAckCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenConfirm(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenConfirmCall { msg: data.0.into() },
                ),
                Msg::ChannelOpenInit(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenInitCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenTry(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenTryCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenAck(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenAckCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenConfirm(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenConfirmCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::RecvPacket(data) => mk_function_call(
                    ibc_handler,
                    RecvPacketCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::AckPacket(data) => mk_function_call(
                    ibc_handler,
                    AcknowledgePacketCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::CreateClient(data) => {
                    let register_client_result = ibc_handler.register_client(
                        data.config.client_type.clone(),
                        data.config.cometbls_client_address.clone().into(),
                    );

                    // TODO(benluelo): Better way to check if client type has already been registered?
                    match register_client_result.send().await {
                        Ok(ok) => {
                            ok.await.unwrap().unwrap();
                        }
                        Err(why) => tracing::info!(
                            "error registering client type, it is likely already registered: {}",
                            why.decode_revert::<String>().unwrap()
                        ),
                    }

                    mk_function_call(
                        ibc_handler,
                        CreateClientCall {
                            msg: contracts::shared_types::MsgCreateClient {
                                client_type: data.config.client_type,
                                client_state_bytes: data.msg.client_state.into_proto_bytes().into(),
                                consensus_state_bytes: data
                                    .msg
                                    .consensus_state
                                    .into_proto_bytes()
                                    .into(),
                            },
                        },
                    )
                }
                Msg::UpdateClient(data) => mk_function_call(
                    ibc_handler,
                    UpdateClientCall {
                        msg: ibc_handler::MsgUpdateClient {
                            client_id: data.msg.client_id.to_string(),
                            client_message: encode_dynamic_singleton_tuple(
                                data.msg.client_message.clone().into_eth_abi(),
                            )
                            .into(),
                        },
                    },
                ),
            };
            let result = msg.send().await;
            match result {
                Ok(ok) => {
                    let tx_rcp = ok.await?.ok_or(TxSubmitError::NoTxReceipt)?;
                    tracing::info!(?tx_rcp, "evm transaction submitted");
                    Ok(())
                }
                Err(ContractError::Revert(revert)) => {
                    tracing::error!(?revert, "evm transaction failed");
                    Ok(())
                }
                _ => {
                    panic!("evm transaction non-recoverable failure");
                }
            }
        })
        .await
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Contract(#[from] ContractError<CometblsMiddleware>),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("no tx receipt from tx")]
    NoTxReceipt,
}

impl MaybeRecoverableError for TxSubmitError {
    fn is_recoverable(&self) -> bool {
        // TODO: Figure out if any failures are unrecoverable
        true
    }
}

fn mk_function_call<Call: EthCall>(
    ibc_handler: IBCHandler<CometblsMiddleware>,
    data: Call,
) -> ethers::contract::FunctionCall<Arc<CometblsMiddleware>, CometblsMiddleware, ()> {
    ibc_handler
        .method_hash(<Call as EthCall>::selector(), data)
        .expect("method selector is generated; qed;")
}

async fn query_client_state<C: ChainSpec>(
    evm: &Evm<C>,
    client_id: chain_utils::evm::EvmClientId,
    height: Height,
) -> Any<
    wasm::client_state::ClientState<
        unionlabs::ibc::lightclients::cometbls::client_state::ClientState,
    >,
> {
    let execution_height = evm.execution_height(height).await;

    let (client_state_bytes, is_found) = evm
        .readonly_ibc_handler
        .get_client_state(client_id.to_string())
        .block(execution_height)
        .await
        .unwrap();

    assert!(is_found);

    Any::try_from_proto_bytes(&client_state_bytes).unwrap()
}

async fn do_fetch<C, L>(evm: &Evm<C>, msg: CometblsFetchMsg<L, C>) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Fetch = CometblsFetchMsg<L, C>, Data = CometblsDataMsg<C>>,
    LightClientSpecificData<L>: From<CometblsDataMsg<C>>,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    let msg = match msg {
        CometblsFetchMsg::FetchFinalityUpdate(PhantomData {}) => CometblsDataMsg::FinalityUpdate(
            FinalityUpdate(evm.beacon_api_client.finality_update().await.unwrap().data),
        ),
        CometblsFetchMsg::FetchLightClientUpdates(FetchLightClientUpdates {
            trusted_period,
            target_period,
            __marker: PhantomData,
        }) => CometblsDataMsg::LightClientUpdates(LightClientUpdates(
            evm.beacon_api_client
                .light_client_updates(trusted_period + 1, target_period - trusted_period)
                .await
                .unwrap()
                .0
                .into_iter()
                .map(|x| x.data)
                .collect(),
        )),
        CometblsFetchMsg::FetchLightClientUpdate(FetchLightClientUpdate {
            period,
            __marker: PhantomData,
        }) => CometblsDataMsg::LightClientUpdate(
            evm.beacon_api_client
                .light_client_updates(period, 1)
                .await
                .unwrap()
                .0
                .into_iter()
                .map(|x| x.data)
                .collect::<Vec<LightClientUpdate<_>>>()
                .pop()
                .unwrap(),
        ),
        CometblsFetchMsg::FetchBootstrap(FetchBootstrap { slot, __marker: _ }) => {
            // NOTE(benluelo): While this is technically two actions, I consider it to be one
            // action - if the beacon chain doesn't have the header, it won't have the bootstrap
            // either. It would be nice if the beacon chain exposed "fetch bootstrap by slot"
            // functionality; I'm surprised it doesn't.

            let mut amount_of_slots_back: u64 = 0;

            let floored_slot = slot
                / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64)
                * (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64);

            tracing::info!("fetching bootstrap at {}", floored_slot);

            let bootstrap = loop {
                let header_response = evm
                    .beacon_api_client
                    .header(beacon_api::client::BlockId::Slot(
                        floored_slot - amount_of_slots_back,
                    ))
                    .await;

                let header = match header_response {
                    Ok(header) => header,
                    Err(beacon_api::errors::Error::NotFound(NotFoundError {
                        status_code: _,
                        error: _,
                        message,
                    })) if message.starts_with("No block found for id") => {
                        amount_of_slots_back += 1;
                        continue;
                    }

                    Err(err) => panic!("{err}"),
                };

                let bootstrap_response = evm
                    .beacon_api_client
                    .bootstrap(header.data.root.clone())
                    .await;

                match bootstrap_response {
                    Ok(ok) => break ok.data,
                    Err(err) => match err {
                        beacon_api::errors::Error::Internal(InternalServerError {
                            status_code: _,
                            error: _,
                            message,
                        }) if message.starts_with("syncCommitteeWitness not available") => {
                            amount_of_slots_back += 1;
                        }
                        _ => panic!("{err}"),
                    },
                };
            };

            // bootstrap contains the current sync committee for the given height
            CometblsDataMsg::Bootstrap(BootstrapData { slot, bootstrap })
        }
        CometblsFetchMsg::FetchAccountUpdate(FetchAccountUpdate { slot, __marker: _ }) => {
            let execution_height = evm
                .execution_height(Height {
                    revision_number: EVM_REVISION_NUMBER,
                    revision_height: slot,
                })
                .await;

            CometblsDataMsg::AccountUpdate(AccountUpdateData {
                slot,
                ibc_handler_address: evm.readonly_ibc_handler.address().0.into(),
                update: evm
                    .provider
                    .get_proof(
                        evm.readonly_ibc_handler.address(),
                        vec![],
                        // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                        Some(execution_height.into()),
                    )
                    .await
                    .unwrap(),
                __marker: PhantomData,
            })
        }
        CometblsFetchMsg::FetchBeaconGenesis(_) => {
            CometblsDataMsg::BeaconGenesis(BeaconGenesisData {
                genesis: evm.beacon_api_client.genesis().await.unwrap().data,
                __marker: PhantomData,
            })
        }
        CometblsFetchMsg::FetchGetProof(get_proof) => {
            let execution_height = evm.execution_height(get_proof.height).await;

            let path = get_proof.path.to_string();

            let location = keccak256(
                keccak256(path.as_bytes())
                    .into_iter()
                    .chain(U256::from(0).encode())
                    .collect::<Vec<_>>(),
            );

            let proof = evm
                .provider
                .get_proof(
                    evm.readonly_ibc_handler.address(),
                    vec![location.into()],
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            tracing::info!(?proof);

            let proof = match <[_; 1]>::try_from(proof.storage_proof) {
                Ok([proof]) => proof,
                Err(invalid) => {
                    panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                }
            };

            let proof = ethereum_v1::StorageProof {
                proofs: [ethereum_v1::Proof {
                    key: proof.key.to_fixed_bytes().to_vec(),
                    // REVIEW(benluelo): Make sure this encoding works
                    value: proof.value.encode(),
                    proof: proof
                        .proof
                        .into_iter()
                        .map(|bytes| bytes.to_vec())
                        .collect(),
                }]
                .to_vec(),
            }
            .encode_to_vec();

            return [match get_proof.path {
                unionlabs::proof::Path::ClientStatePath(_) => data::<L>(
                    evm.chain_id,
                    ClientStateProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ClientConsensusStatePath(_) => data::<L>(
                    evm.chain_id,
                    ClientConsensusStateProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ConnectionPath(_) => data::<L>(
                    evm.chain_id,
                    ConnectionProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ChannelEndPath(_) => data::<L>(
                    evm.chain_id,
                    ChannelEndProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::CommitmentPath(_) => data::<L>(
                    evm.chain_id,
                    CommitmentProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::AcknowledgementPath(_) => data::<L>(
                    evm.chain_id,
                    AcknowledgementProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
            }]
            .into();
        }
    };

    [data::<L>(evm.chain_id, LightClientSpecificData::from(msg))].into()
}

impl<Counterparty, C, P> IbcStateRead<Counterparty, P> for Evm<C>
where
    Counterparty: Chain,
    C: ChainSpec,
    P: IbcPath<Evm<C>, Counterparty>
        + EthereumStateRead<
            C,
            Counterparty,
            Encoded = <<<P as EthereumStateRead<C, Counterparty>>::EthCall as EthCallExt>::Return as TupleToOption>::Inner,
        > + 'static,
    <P::EthCall as EthCallExt>::Return: TupleToOption,
{
    fn state_proof(
        &self,
        path: P,
        at: Height,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let execution_height = self.execution_height(at).await;

            let path = path.to_string();

            let location = keccak256(
                keccak256(path.as_bytes())
                    .into_iter()
                    .chain(U256::from(0).encode())
                    .collect::<Vec<_>>(),
            );

            let proof = self
                .provider
                .get_proof(
                    self.readonly_ibc_handler.address(),
                    vec![location.into()],
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            tracing::info!(?proof);

            let proof = match <[_; 1]>::try_from(proof.storage_proof) {
                Ok([proof]) => proof,
                Err(invalid) => {
                    panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                }
            };

                ethereum_v1::StorageProof {
                    proofs: [ethereum_v1::Proof {
                        key: proof.key.to_fixed_bytes().to_vec(),
                        // REVIEW(benluelo): Make sure this encoding works
                        value: proof.value.encode(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    }]
                    .to_vec(),
                }
                .encode_to_vec()
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct GetProof<C: ChainSpec, L: LightClient<HostChain = Evm<C>>> {
    path: unionlabs::proof::Path<
        <L::HostChain as Chain>::ClientId,
        <ChainOf<L::Counterparty> as Chain>::Height,
    >,
    height: <Evm<C> as Chain>::Height,
}

// pub struct AnyGetProof<C: ChainSpec, L: LightClient<HostChain = Evm<C>>> {
//     __marker: PhantomData<fn() -> (C, L)>,
// }

// type _AnyGetProof<C, L> = Path2<L, AnyGetProof<C, L>>;

trait EthereumStateRead<C, Counterparty>: IbcPath<Evm<C>, Counterparty>
where
    Counterparty: Chain,
    C: ChainSpec,
{
    /// The type of the encoded state returned from the contract. This may be bytes (see client state)
    /// or a type (see connection end)
    /// Since solidity doesn't support generics, it emulates generics by using bytes in interfaces and
    /// "downcasting" (via parsing) to expected types in implementations.
    type Encoded;

    type EthCall: EthCallExt + 'static;

    fn into_eth_call(self) -> Self::EthCall;

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output;
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty>
    for ClientStatePath<<Evm<C> as Chain>::ClientId>
where
    ClientStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ClientStateOf<Counterparty>>: Debug,
{
    type Encoded = Vec<u8>;

    type EthCall = GetClientStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        TryFromProto::try_from_proto_bytes(&encoded).unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty>
    for ClientConsensusStatePath<<Evm<C> as Chain>::ClientId, <Counterparty as Chain>::Height>
where
    ConsensusStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ConsensusStateOf<Counterparty>>: Debug,
{
    type Encoded = Vec<u8>;

    type EthCall = GetConsensusStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
            height: self.height.into_height().into(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        TryFromProto::try_from_proto_bytes(&encoded).unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for ConnectionPath {
    type Encoded = IbcCoreConnectionV1ConnectionEndData;

    type EthCall = GetConnectionCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            connection_id: self.connection_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.try_into().unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for ChannelEndPath {
    type Encoded = IbcCoreChannelV1ChannelData;

    type EthCall = GetChannelCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.try_into().unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for CommitmentPath {
    type Encoded = [u8; 32];

    type EthCall = GetHashedPacketCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence,
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.into()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for AcknowledgementPath {
    type Encoded = [u8; 32];

    type EthCall = GetHashedPacketAcknowledgementCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence,
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.into()
    }
}

// fn decode_log<T: EthLogDecode + Debug>(logs: impl IntoIterator<Item = impl Into<RawLog>>) -> T {
//     let t = decode_logs::<T>(&logs.into_iter().map(Into::into).collect::<Vec<_>>()).unwrap();

//     let [t] = <[T; 1]>::try_from(t)
//         .map_err(|err| format!("invalid events, expected one event but got {err:#?}"))
//         .unwrap();

//     t
// }

impl<L, C> UseAggregate<L> for Identified<L, CreateUpdateData<L, C>>
where
    Identified<L, AccountUpdateData<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, LightClientUpdate<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, BeaconGenesisData<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,

    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L::Counterparty>)>,

    L: LightClient<HostChain = Evm<C>>,
    C: ChainSpec,
{
    type AggregatedData = HList![
        Identified<L, LightClientUpdate<C>>,
        Identified<L, AccountUpdateData<C>>,
        Identified<L, BeaconGenesisData<C>>
    ];

    fn aggregate(
        Identified {
            chain_id,
            data:
                CreateUpdateData {
                    req,
                    currently_trusted_slot,
                    light_client_update,
                    is_next,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: bootstrap_chain_id,
                data: LightClientUpdate {
                    next_sync_committee,
                    ..
                }
            },
            Identified {
                chain_id: account_update_chain_id,
                data: AccountUpdateData {
                    slot: account_update_data_beacon_slot,
                    ibc_handler_address,
                    update: account_update,
                    __marker,
                }
            },
            Identified {
                chain_id: beacon_api_chain_id,
                data: BeaconGenesisData {
                    genesis,
                    __marker: _,
                }
            }
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(bootstrap_chain_id, account_update_chain_id);
        assert_eq!(chain_id, account_update_chain_id);
        assert_eq!(chain_id, beacon_api_chain_id);

        let header = wasm::header::Header {
            height: Height {
                revision_number: EVM_REVISION_NUMBER,
                revision_height: account_update_data_beacon_slot,
            },
            data: ethereum::header::Header {
                consensus_update: light_client_update,
                trusted_sync_committee: TrustedSyncCommittee {
                    trusted_height: Height {
                        revision_number: EVM_REVISION_NUMBER,
                        revision_height: currently_trusted_slot,
                    },
                    sync_committee: if is_next {
                        ActiveSyncCommittee::Next(next_sync_committee.unwrap())
                    } else {
                        ActiveSyncCommittee::Current(next_sync_committee.unwrap())
                    },
                },
                account_update: AccountUpdate {
                    proofs: [Proof {
                        key: ibc_handler_address.into(),
                        value: account_update.storage_hash.as_bytes().to_vec(),
                        proof: account_update
                            .account_proof
                            .into_iter()
                            .map(|x| x.to_vec())
                            .collect(),
                    }]
                    .to_vec(),
                },
            },
        };

        seq([
            wait::<L::Counterparty>(
                req.counterparty_chain_id.clone(),
                WaitForTimestamp {
                    timestamp: (genesis.genesis_time
                        + (header.data.consensus_update.signature_slot * C::SECONDS_PER_SLOT::U64))
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                },
            ),
            crate::msg::msg::<L::Counterparty>(
                req.counterparty_chain_id,
                MsgUpdateClientData {
                    msg: MsgUpdateClient {
                        client_id: req.counterparty_client_id,
                        client_message: header,
                    },
                    update_from: Height {
                        revision_number: 0,
                        revision_height: currently_trusted_slot,
                    },
                },
            ),
        ])
    }
}

impl<L, C> UseAggregate<L> for Identified<L, MakeCreateUpdatesData<L, C>>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    Identified<L, FinalityUpdate<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AggregateReceiver: From<Identified<L, Aggregate<L>>>,
{
    type AggregatedData = HList![
        Identified<L, FinalityUpdate<C>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            data: MakeCreateUpdatesData { req },
        }: Self,
        hlist_pat![Identified {
            chain_id: bootstrap_chain_id,
            data: FinalityUpdate(finality_update)
        },]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, bootstrap_chain_id);

        let target_period =
            sync_committee_period::<_, C>(finality_update.attested_header.beacon.slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        assert!(
            trusted_period <= target_period,
            "trusted period {trusted_period} is behind target period {target_period}, something is wrong!",
        );

        // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
        // updates until we reach the `target_period - 1`.
        RelayerMsg::Aggregate {
            queue: [fetch::<L>(
                chain_id,
                LightClientSpecificFetch(CometblsFetchMsg::FetchLightClientUpdates(
                    FetchLightClientUpdates {
                        trusted_period,
                        target_period,
                        __marker: PhantomData,
                    },
                )),
            )]
            .into(),
            data: [].into(),
            receiver: AggregateReceiver::from(Identified {
                chain_id,
                data: Aggregate::LightClientSpecific(LightClientSpecificAggregate(
                    CometblsAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(
                        MakeCreateUpdatesFromLightClientUpdatesData {
                            req: req.clone(),
                            trusted_height: req.update_from,
                            finality_update,
                        },
                    ),
                )),
            }),
        }
    }
}

impl<L, C> UseAggregate<L> for Identified<L, MakeCreateUpdatesFromLightClientUpdatesData<L, C>>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,

    Identified<L, LightClientUpdates<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,

    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
    AggregateReceiver: From<Identified<L, Aggregate<L>>>,
{
    type AggregatedData = HList![
        Identified<L, LightClientUpdates<C>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            data:
                MakeCreateUpdatesFromLightClientUpdatesData {
                    req,
                    trusted_height,
                    finality_update,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: light_client_updates_chain_id,
            data: LightClientUpdates(light_client_updates)
        },]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, light_client_updates_chain_id);

        let target_period = sync_committee_period::<_, C>(finality_update.signature_slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        let (updates, last_update_block_number) = light_client_updates.into_iter().fold(
            (VecDeque::new(), trusted_height.revision_height),
            |(mut vec, mut trusted_slot), update| {
                let old_trusted_slot = trusted_slot;

                trusted_slot = update.attested_header.beacon.slot;

                vec.push_back(make_create_update::<C, L>(
                    req.clone(),
                    chain_id,
                    old_trusted_slot,
                    update,
                    true,
                ));

                (vec, trusted_slot)
            },
        );

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let finality_update_attested_header_slot = finality_update.attested_header.beacon.slot;

        let does_not_have_finality_update =
            last_update_block_number >= req.update_to.revision_height;

        tracing::error!(last_update_block_number, req.update_to.revision_height);

        let finality_update_msg = if does_not_have_finality_update {
            // do nothing
            None
        } else {
            // do finality update
            Some(make_create_update(
                req.clone(),
                chain_id,
                last_update_block_number,
                LightClientUpdate {
                    attested_header: finality_update.attested_header,
                    next_sync_committee: None,
                    next_sync_committee_branch: None,
                    finalized_header: finality_update.finalized_header,
                    finality_branch: finality_update.finality_branch,
                    sync_aggregate: finality_update.sync_aggregate,
                    signature_slot: finality_update.signature_slot,
                },
                false,
            ))
        };

        seq(lc_updates
            .into_iter()
            .chain(finality_update_msg)
            .chain([fetch::<L>(
                chain_id,
                FetchTrustedClientState {
                    at: QueryHeight::Specific(Height {
                        revision_number: EVM_REVISION_NUMBER,
                        revision_height: (!does_not_have_finality_update)
                            .then_some(finality_update_attested_header_slot)
                            .unwrap_or_else(|| {
                                std::cmp::max(
                                    req.update_to.revision_height,
                                    last_update_block_number,
                                )
                            }),
                    }),
                    client_id: req.client_id,
                },
            )]))
    }
}
