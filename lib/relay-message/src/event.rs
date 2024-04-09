use std::marker::PhantomData;

use chain_utils::GetChain;
use macros::apply;
use queue_msg::{
    aggregate, fetch, queue_msg, wait, HandleEvent, QueueError, QueueMsg, QueueMsgTypes,
};
use unionlabs::{
    hash::H256,
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath},
    traits::{ClientIdOf, ClientTypeOf, HeightOf},
};

use crate::{
    aggregate::{
        mk_aggregate_wait_for_update, Aggregate, AggregateChannelHandshakeMsgAfterUpdate,
        AggregateConnectionFetchFromChannelEnd, AggregateConnectionOpenAck,
        AggregateConnectionOpenConfirm, AggregateConnectionOpenTry, AggregateMsgAfterUpdate,
        AggregatePacketMsgAfterUpdate, AggregateUpdateClient, AnyAggregate, ChannelHandshakeEvent,
        PacketEvent,
    },
    any_enum, any_lc,
    fetch::{AnyFetch, Fetch, FetchLatestClientState, FetchState},
    id, identified, seq,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, RelayMessageTypes,
};

#[apply(any_enum)]
#[any = AnyEvent]
pub enum Event<Hc: ChainExt, Tr: ChainExt> {
    Ibc(IbcEvent<Hc, Tr>),
    Command(Command<Hc, Tr>),
}

impl HandleEvent<RelayMessageTypes> for AnyLightClientIdentified<AnyEvent> {
    fn handle(
        self,
        store: &<RelayMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<RelayMessageTypes>, QueueError> {
        let wait = self;

        any_lc! {
            |wait| {
                store
                    .with_chain(&wait.chain_id, move |c| wait.t.handle(c))
                    .map_err(|e| QueueError::Fatal(Box::new(e)))
            }
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Event<Hc, Tr> {
    pub fn handle(self, hc: Hc) -> QueueMsg<RelayMessageTypes>
    where
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
    {
        match self {
            Event::Ibc(ibc_event) => match ibc_event.event {
                unionlabs::events::IbcEvent::CreateClient(e) => {
                    tracing::info!("client created: {e:?}");

                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::UpdateClient(e) => {
                    tracing::info!(
                        "client updated: {} to {:?}",
                        e.client_id,
                        e.consensus_heights
                    );

                    QueueMsg::Noop
                }

                unionlabs::events::IbcEvent::ClientMisbehaviour(_) => unimplemented!(),
                unionlabs::events::IbcEvent::SubmitEvidence(_) => unimplemented!(),

                unionlabs::events::IbcEvent::ConnectionOpenInit(init) => seq([
                    wait(id(
                        hc.chain_id(),
                        WaitForBlock {
                            height: ibc_event.height,
                            __marker: PhantomData,
                        },
                    )),
                    aggregate(
                        [mk_aggregate_wait_for_update(
                            hc.chain_id(),
                            init.client_id.clone(),
                            init.counterparty_client_id.clone(),
                            ibc_event.height,
                        )],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateMsgAfterUpdate::ConnectionOpenTry(
                                AggregateConnectionOpenTry {
                                    event_height: ibc_event.height,
                                    event: init,
                                },
                            ),
                        ),
                    ),
                ]),
                unionlabs::events::IbcEvent::ConnectionOpenTry(try_) => aggregate(
                    [mk_aggregate_wait_for_update(
                        hc.chain_id(),
                        try_.client_id.clone(),
                        try_.counterparty_client_id.clone(),
                        ibc_event.height,
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateMsgAfterUpdate::ConnectionOpenAck(AggregateConnectionOpenAck {
                            event_height: ibc_event.height,
                            event: try_,
                        }),
                    ),
                ),
                unionlabs::events::IbcEvent::ConnectionOpenAck(ack) => aggregate(
                    [mk_aggregate_wait_for_update(
                        hc.chain_id(),
                        ack.client_id.clone(),
                        ack.counterparty_client_id.clone(),
                        ibc_event.height,
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateMsgAfterUpdate::ConnectionOpenConfirm(
                            AggregateConnectionOpenConfirm {
                                event_height: ibc_event.height,
                                event: ack,
                            },
                        ),
                    ),
                ),
                unionlabs::events::IbcEvent::ConnectionOpenConfirm(confirm) => {
                    tracing::info!("connection opened: {confirm:?}");

                    QueueMsg::Noop
                }

                unionlabs::events::IbcEvent::ChannelOpenInit(init) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: init.port_id.clone(),
                                    channel_id: init.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateConnectionFetchFromChannelEnd {
                                at: ibc_event.height,
                                __marker: PhantomData,
                            },
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeMsgAfterUpdate {
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Init(init),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenTry(try_) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: try_.port_id.clone(),
                                    channel_id: try_.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateConnectionFetchFromChannelEnd {
                                at: ibc_event.height,
                                __marker: PhantomData,
                            },
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeMsgAfterUpdate {
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Try(try_),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenAck(ack) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: ack.port_id.clone(),
                                    channel_id: ack.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateConnectionFetchFromChannelEnd {
                                at: ibc_event.height,
                                __marker: PhantomData,
                            },
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeMsgAfterUpdate {
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Ack(ack),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenConfirm(confirm) => {
                    tracing::info!("channel opened: {confirm:?}");

                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::RecvPacket(packet) => {
                    // in parallel, run height timeout, timestamp timeout, and send packet
                    let aggregate_timeout_packet: QueueMsg<RelayMessageTypes> = aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: QueryHeight::Specific(ibc_event.height),
                                path: ConnectionPath {
                                    connection_id: send.connection_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregatePacketMsgAfterUpdate {
                                update_to: ibc_event.height,
                                event_height: ibc_event.height,
                                tx_hash: ibc_event.tx_hash,
                                packet_event: PacketEvent::Timeout(Packet {
                                    sequence: send.packet_sequence,
                                    source_port: send.packet_src_port.clone(),
                                    source_channel: send.packet_src_channel.clone(),
                                    destination_port: send.packet_dst_port.clone(),
                                    destination_channel: send.packet_dst_channel.clone(),
                                    data: send.packet_data_hex.clone(),
                                    timeout_height: send.packet_timeout_height,
                                    timeout_timestamp: send.packet_timeout_timestamp,
                                }),
                                __marker: PhantomData,
                            },
                        ),
                    );

                    conc(
                        [
                            (send.packet_timeout_height != Height::default()).then(|| {
                                seq([
                                    wait(id(
                                        hc.chain_id(),
                                        WaitForHeight {
                                            height: send.packet_timeout_height.into(),
                                            __marker: PhantomData,
                                        },
                                    )),
                                    aggregate_timeout_packet.clone(),
                                ])
                            }),
                            (send.packet_timeout_timestamp != 0).then(|| {
                                seq([
                                    wait(id(
                                        hc.chain_id(),
                                        WaitForTimestamp {
                                            timestamp: send
                                                .packet_timeout_timestamp
                                                .try_into()
                                                .unwrap(),
                                            __marker: PhantomData,
                                        },
                                    )),
                                    aggregate_timeout_packet,
                                ])
                            }),
                            Some(aggregate(
                                [fetch(id::<Hc, Tr, _>(
                                    hc.chain_id(),
                                    FetchState {
                                        at: QueryHeight::Specific(ibc_event.height),
                                        path: ConnectionPath {
                                            connection_id: send.connection_id.clone(),
                                        }
                                        .into(),
                                    },
                                ))],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregatePacketMsgAfterUpdate {
                                        update_to: ibc_event.height,
                                        event_height: ibc_event.height,
                                        tx_hash: ibc_event.tx_hash,
                                        packet_event: PacketEvent::Send(send),
                                        __marker: PhantomData,
                                    },
                                ),
                            )),
                        ]
                        .into_iter()
                        .flatten(),
                    );

                    aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ConnectionPath {
                                    connection_id: packet.connection_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregatePacketMsgAfterUpdate {
                                update_to: ibc_event.height,
                                event_height: ibc_event.height,
                                tx_hash: ibc_event.tx_hash,
                                packet_event: PacketEvent::Recv(packet),
                                __marker: PhantomData,
                            },
                        ),
                    )
                }
                unionlabs::events::IbcEvent::SendPacket(packet) => aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchState {
                            at: ibc_event.height,
                            path: ConnectionPath {
                                connection_id: packet.connection_id.clone(),
                            }
                            .into(),
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregatePacketMsgAfterUpdate {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            tx_hash: ibc_event.tx_hash,
                            packet_event: PacketEvent::Send(packet),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::AcknowledgePacket(ack) => {
                    tracing::info!(?ack, "packet acknowledged");
                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::TimeoutPacket(timeout) => {
                    tracing::error!(?timeout, "packet timed out");
                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::WriteAcknowledgement(write_ack) => {
                    tracing::info!(?write_ack, "packet acknowledgement written");
                    QueueMsg::Noop
                }
            },
            Event::Command(command) => match command {
                Command::UpdateClient {
                    client_id,
                    __marker: _,
                } => aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchLatestClientState {
                            path: ClientStatePath {
                                client_id: client_id.clone(),
                            },
                            __marker: PhantomData,
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateUpdateClient {
                            client_id,
                            __marker: PhantomData,
                        },
                    ),
                ),
            },
        }
    }
}

#[queue_msg]
pub struct IbcEvent<Hc: ChainExt, Tr: ChainExt> {
    pub tx_hash: H256,
    pub height: HeightOf<Hc>,
    pub event: unionlabs::events::IbcEvent<ClientIdOf<Hc>, ClientTypeOf<Hc>, ClientIdOf<Tr>>,
}

#[queue_msg]
pub enum Command<Hc: ChainExt, Tr: ChainExt> {
    UpdateClient {
        client_id: ClientIdOf<Hc>,
        #[serde(skip)]
        __marker: PhantomData<fn() -> Tr>,
    },
}
