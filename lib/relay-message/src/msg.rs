use std::{fmt::Display, marker::PhantomData};

use chain_utils::GetChain;
use macros::apply;
use queue_msg::{msg_struct, HandleMsg, QueueError, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::{
        channel::{
            msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket,
        },
        client::{msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient},
        connection::{
            msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_init::MsgConnectionOpenInit,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    id::ConnectionId,
    traits::{ClientIdOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
};

use crate::{any_enum, any_lc, AnyLightClientIdentified, ChainExt, DoMsg, RelayerMsgTypes};

#[apply(any_enum)]
#[any = AnyMsg]
pub enum Msg<Hc: ChainExt, Tr: ChainExt> {
    ConnectionOpenInit(MsgConnectionOpenInitData<Hc, Tr>),
    ConnectionOpenTry(MsgConnectionOpenTryData<Hc, Tr>),
    ConnectionOpenAck(MsgConnectionOpenAckData<Hc, Tr>),
    ConnectionOpenConfirm(MsgConnectionOpenConfirmData<Hc, Tr>),

    ChannelOpenInit(MsgChannelOpenInitData<Hc, Tr>),
    ChannelOpenTry(MsgChannelOpenTryData<Hc, Tr>),
    ChannelOpenAck(MsgChannelOpenAckData<Hc, Tr>),
    ChannelOpenConfirm(MsgChannelOpenConfirmData<Hc, Tr>),

    RecvPacket(MsgRecvPacketData<Hc, Tr>),
    AckPacket(MsgAckPacketData<Hc, Tr>),

    CreateClient(MsgCreateClientData<Hc, Tr>),
    UpdateClient(MsgUpdateClientData<Hc, Tr>),
}

impl HandleMsg<RelayerMsgTypes> for AnyLightClientIdentified<AnyMsg> {
    async fn handle(
        self,
        store: &<RelayerMsgTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<RelayerMsgTypes>, QueueError> {
        let msg = self;

        any_lc! {
            |msg| {
                store
                    .with_chain(&msg.chain_id, move |c| async move { msg.t.handle(&c).await })
                    .map_err(|e| QueueError::Fatal(Box::new(e)))?
                    .await
                    .map_err(|e: <Hc as ChainExt>::MsgError| QueueError::Retry(Box::new(e)))
                    .map(|()| QueueMsg::Noop)
            }
        }
    }
}

impl<Hc, Tr> Msg<Hc, Tr>
where
    Hc: ChainExt + DoMsg<Hc, Tr>,
    Tr: ChainExt,
{
    pub async fn handle(self, c: &Hc) -> Result<(), Hc::MsgError> {
        <Hc as DoMsg<Hc, Tr>>::msg(c, self).await
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Msg<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Msg::ConnectionOpenInit(_) => write!(f, "ConnectionOpenInit"),
            Msg::ConnectionOpenTry(_) => write!(f, "ConnectionOpenTry"),
            Msg::ConnectionOpenAck(_) => write!(f, "ConnectionOpenAck"),
            Msg::ConnectionOpenConfirm(_) => write!(f, "ConnectionOpenConfirm"),
            Msg::ChannelOpenInit(_) => write!(f, "ChannelOpenInit"),
            Msg::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
            Msg::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
            Msg::ChannelOpenConfirm(_) => write!(f, "ChannelOpenConfirm"),
            Msg::RecvPacket(_) => write!(f, "RecvPacket"),
            Msg::AckPacket(_) => write!(f, "AckPacket"),
            Msg::CreateClient(_) => write!(f, "CreateClient"),
            Msg::UpdateClient(_) => write!(f, "UpdateClient"),
        }
    }
}

#[apply(msg_struct)]
pub struct MsgConnectionOpenInitData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
);

#[apply(msg_struct)]
pub struct MsgConnectionOpenTryData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenTry<
        Tr::StoredClientState<Hc>,
        ClientIdOf<Hc>,
        ClientIdOf<Tr>,
        ConnectionId,
        Tr::Height,
        Hc::Height,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[apply(msg_struct)]
pub struct MsgConnectionOpenAckData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenAck<
        Tr::StoredClientState<Hc>,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgConnectionOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenConfirm<HeightOf<Tr>, Tr::StateProof>,
}

#[apply(msg_struct)]
#[cover(Hc, Tr)]
pub struct MsgChannelOpenInitData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenInit,
}

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgChannelOpenTryData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenTry<Tr::StateProof>,
}

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgChannelOpenAckData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenAck<Tr::StateProof, Tr::Height>,
}

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgChannelOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenConfirm<Tr::StateProof>,
}

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgRecvPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgRecvPacket<Tr::StateProof, Tr::Height>,
}

#[apply(msg_struct)]
#[cover(Hc)]
pub struct MsgAckPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgAcknowledgement<Tr::StateProof, Tr::Height>,
}

#[apply(msg_struct)]
pub struct MsgCreateClientData<Hc: ChainExt, Tr: ChainExt> {
    pub config: Hc::Config,
    pub msg: MsgCreateClient<ClientStateOf<Tr>, ConsensusStateOf<Tr>>,
}

#[apply(msg_struct)]
pub struct MsgUpdateClientData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgUpdateClient<ClientIdOf<Hc>, HeaderOf<Tr>>,
);
