use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::H256,
    id::{ChannelId, PortId},
    proof::{self, ClientStatePath},
    traits::{ChainIdOf, ClientIdOf, HeightOf},
    QueryHeight,
};

use crate::{
    any_enum, data,
    data::{AnyData, Data, PacketAcknowledgement, SelfClientState, SelfConsensusState},
    fetch, identified, AnyLightClientIdentified, ChainExt, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, RelayerMsg,
};

any_enum! {
    /// Fetch some data that will likely be used in a [`RelayerMsg::Aggregate`].
    #[any = AnyFetch]
    pub enum Fetch<Hc: ChainExt, Tr: ChainExt> {
        State(FetchState<Hc, Tr>),
        Proof(FetchProof<Hc, Tr>),

        LatestClientState(FetchLatestClientState<Hc, Tr>),

        SelfClientState(FetchSelfClientState<Hc, Tr>),
        SelfConsensusState(FetchSelfConsensusState<Hc, Tr>),

        PacketAcknowledgement(FetchPacketAcknowledgement<Hc, Tr>),

        UpdateHeaders(FetchUpdateHeaders<Hc, Tr>),

        LightClientSpecific(LightClientSpecificFetch<Hc, Tr>),
    }
}

pub trait DoFetch<Hc: ChainExt>: Sized + Debug + Clone + PartialEq {
    fn do_fetch(c: &Hc, _: Self) -> impl Future<Output = Vec<RelayerMsg>>;
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Fetch<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path_display = |path: &_| match path {
            proof::Path::ClientStatePath(_) => "ClientState",
            proof::Path::ClientConsensusStatePath(_) => "ClientConsensusState",
            proof::Path::ConnectionPath(_) => "Connection",
            proof::Path::ChannelEndPath(_) => "ChannelEnd",
            proof::Path::CommitmentPath(_) => "Commitment",
            proof::Path::AcknowledgementPath(_) => "Acknowledgement",
        };

        match self {
            Fetch::State(fetch) => write!(f, "State({})", path_display(&fetch.path)),
            Fetch::Proof(fetch) => write!(f, "Proof({})", path_display(&fetch.path)),
            Fetch::LatestClientState(_) => write!(f, "LatestClientState"),
            Fetch::SelfClientState(_) => write!(f, "SelfClientState"),
            Fetch::SelfConsensusState(_) => write!(f, "SelfConsensusState"),
            Fetch::PacketAcknowledgement(_) => write!(f, "PacketAcknowledgement"),
            Fetch::UpdateHeaders(_) => write!(f, "UpdateHeaders"),
            Fetch::LightClientSpecific(fetch) => write!(f, "LightClientSpecific({})", fetch.0),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchSelfClientState<Hc: ChainExt, Tr: ChainExt> {
    pub at: QueryHeight<HeightOf<Hc>>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchSelfConsensusState<Hc: ChainExt, Tr: ChainExt> {
    pub at: QueryHeight<HeightOf<Hc>>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchProof<Hc: ChainExt, Tr: ChainExt> {
    pub at: HeightOf<Hc>,
    pub path: proof::Path<Hc::ClientId, Tr::Height>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchState<Hc: ChainExt, Tr: ChainExt> {
    pub at: HeightOf<Hc>,
    pub path: proof::Path<Hc::ClientId, Tr::Height>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchLatestClientState<Hc: ChainExt, Tr: ChainExt> {
    pub path: ClientStatePath<Hc::ClientId>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchPacketAcknowledgement<Hc: ChainExt, Tr: ChainExt> {
    pub block_hash: H256,
    pub destination_port_id: PortId,
    pub destination_channel_id: ChannelId,
    pub sequence: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt> {
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_chain_id: ChainIdOf<Tr>,
    // id of the counterparty client that will be updated with the fetched headers
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub update_from: HeightOf<Hc>,
    pub update_to: HeightOf<Hc>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificFetch<Hc: ChainExt, Tr: ChainExt>(pub Hc::Fetch<Tr>);

impl<Hc, Tr> Fetch<Hc, Tr>
where
    Hc: ChainExt + DoFetchState<Hc, Tr> + DoFetchProof<Hc, Tr> + DoFetchUpdateHeaders<Hc, Tr>,
    Hc::Fetch<Tr>: DoFetch<Hc>,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
{
    pub async fn handle(self, c: Hc) -> Vec<RelayerMsg> {
        match self {
            Fetch::Proof(msg) => [Hc::proof(&c, msg.at, msg.path)].into(),
            Fetch::State(msg) => [Hc::state(&c, msg.at, msg.path)].into(),
            Fetch::SelfClientState(FetchSelfClientState {
                at: height,
                __marker: _,
            }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => c.query_latest_height().await.unwrap(),
                    QueryHeight::Specific(h) => h,
                };

                [data(
                    c.chain_id(),
                    SelfClientState {
                        self_client_state: c.self_client_state(height).await,
                        __marker: PhantomData,
                    },
                )]
                .into()
            }
            Fetch::SelfConsensusState(FetchSelfConsensusState {
                at: height,
                __marker: _,
            }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => c.query_latest_height().await.unwrap(),
                    QueryHeight::Specific(h) => h,
                };

                [data(
                    c.chain_id(),
                    SelfConsensusState {
                        self_consensus_state: c.self_consensus_state(height).await,
                        __marker: PhantomData,
                    },
                )]
                .into()
            }
            Fetch::PacketAcknowledgement(FetchPacketAcknowledgement {
                block_hash,
                destination_port_id,
                destination_channel_id,
                sequence,
                __marker,
            }) => {
                let ack = c
                    .read_ack(
                        block_hash.clone(),
                        destination_channel_id.clone(),
                        destination_port_id.clone(),
                        sequence,
                    )
                    .await;

                [data(
                    c.chain_id(),
                    PacketAcknowledgement {
                        fetched_by: FetchPacketAcknowledgement {
                            block_hash,
                            destination_port_id,
                            destination_channel_id,
                            sequence,
                            __marker,
                        },
                        ack,
                    },
                )]
                .into()
            }
            Fetch::UpdateHeaders(fetch_update_headers) => {
                [Hc::fetch_update_headers(&c, fetch_update_headers)].into()
            }
            Fetch::LightClientSpecific(LightClientSpecificFetch(fetch)) => c.do_fetch(fetch).await,
            Fetch::LatestClientState(FetchLatestClientState { path, __marker }) => {
                [fetch::<Hc, Tr>(
                    c.chain_id(),
                    FetchState {
                        at: c.query_latest_height().await.unwrap(),
                        path: path.into(),
                    },
                )]
                .into()
            }
        }
    }
}
