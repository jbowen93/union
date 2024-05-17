use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::{
        channel::{self, channel::Channel, packet::Packet},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection,
    },
    ics24::{ChannelEndPath, ConnectionPath, ReceiptPath},
    validated::ValidateT,
};

use super::connection_handshake::ConnectionEnd;
use crate::{Either, IbcError, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvPacket {
    Init {
        packet: Packet,
        proof_commitment: Vec<u8>,
        proof_height: Height,
    },

    MembershipVerified {
        packet: Packet,
        channel: Channel,
    },

    CallbackCalled {
        packet: Packet,
        channel: Channel,
    },
}

impl<T: IbcHost> Runnable<T> for RecvPacket {
    fn process(
        self,
        host: &mut T,
        resp: crate::IbcResponse,
    ) -> Result<crate::Either<(Self, crate::IbcMsg), crate::IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                RecvPacket::Init {
                    packet,
                    proof_commitment,
                    proof_height,
                },
                IbcResponse::Empty,
            ) => {
                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: packet.destination_port.clone(),
                            channel_id: packet.destination_channel.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(
                        packet.destination_port.clone(),
                        packet.destination_channel.clone(),
                    ))?;

                if channel.state != channel::state::State::Open {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Open,
                    )
                    .into());
                }

                if packet.source_port != channel.counterparty.port_id {
                    return Err(IbcError::PortMismatch(
                        packet.source_port,
                        channel.counterparty.port_id,
                    )
                    .into());
                }

                if packet.source_channel.to_string() != channel.counterparty.channel_id {
                    return Err(IbcError::ChannelMismatch(
                        packet.source_channel,
                        channel.counterparty.channel_id.validate().unwrap(),
                    )
                    .into());
                }

                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: channel.connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(
                        channel.connection_hops[0].to_string(),
                    ))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                if packet.timeout_height > host.current_height() {
                    return Err(IbcError::TimedOutPacket.into());
                }

                if packet.timeout_timestamp > host.current_timestamp() {
                    return Err(IbcError::TimedOutPacket.into());
                }

                // TODO(aeryz): recv start sequence check for replay protection

                match host.read_raw(&format!(
                    "receipts/ports/{}/channels/{}/sequences/{}",
                    packet.destination_port, packet.destination_channel, packet.sequence
                )) {
                    Some(_) => Either::Right(IbcEvent::RecvPacket {
                        packet_data_hex: packet.data,
                        packet_timeout_height: packet.timeout_height,
                        packet_timeout_timestamp: packet.timeout_timestamp,
                        packet_sequence: packet.sequence,
                        packet_src_port: packet.source_port,
                        packet_src_channel: packet.source_channel,
                        packet_dst_port: packet.destination_port,
                        packet_dst_channel: packet.destination_channel,
                        packet_channel_ordering: channel.ordering,
                        connection_id: channel.connection_hops[0].clone(),
                    }),
                    None => {
                        // TODO(aeryz): known size can be optimized
                        let mut packet_commitment = Vec::new();
                        packet_commitment
                            .extend_from_slice(packet.timeout_timestamp.to_be_bytes().as_slice());
                        packet_commitment.extend_from_slice(
                            packet
                                .timeout_height
                                .revision_number
                                .to_be_bytes()
                                .as_slice(),
                        );
                        packet_commitment.extend_from_slice(
                            packet
                                .timeout_height
                                .revision_height
                                .to_be_bytes()
                                .as_slice(),
                        );
                        packet_commitment
                            .extend_from_slice(host.sha256(packet.data.clone()).as_slice());

                        Either::Left((
                            RecvPacket::MembershipVerified {
                                packet: packet.clone(),
                                channel: channel.clone(),
                            },
                            IbcMsg::VerifyMembership {
                                client_id: connection.client_id,
                                height: proof_height,
                                delay_time_period: 0,
                                delay_block_period: 0,
                                proof: proof_commitment,
                                path: MerklePath {
                                    key_path: vec![
                                        "ibc".into(),
                                        format!(
                                            "commitmens/ports/{}/channels/{}/sequences/{}",
                                            packet.source_port,
                                            packet.source_channel,
                                            packet.sequence
                                        ),
                                    ],
                                },
                                value: host.sha256(packet_commitment),
                            },
                        ))
                    }
                }
            }
            (
                RecvPacket::MembershipVerified { packet, channel },
                IbcResponse::VerifyMembership { valid },
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                Either::Left((
                    RecvPacket::CallbackCalled {
                        packet: packet.clone(),
                        channel,
                    },
                    IbcMsg::OnRecvPacket {
                        packet: packet.clone(),
                    },
                ))
            }
            (RecvPacket::CallbackCalled { packet, channel }, IbcResponse::OnRecvPacket { err }) => {
                if err {
                    return Err(IbcError::IbcAppCallbackFailed.into());
                }

                host.commit_raw(
                    ReceiptPath {
                        port_id: packet.destination_port.clone(),
                        channel_id: packet.destination_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                    vec![1],
                )?;

                Either::Right(IbcEvent::RecvPacket {
                    packet_data_hex: packet.data,
                    packet_timeout_height: packet.timeout_height,
                    packet_timeout_timestamp: packet.timeout_timestamp,
                    packet_sequence: packet.sequence,
                    packet_src_port: packet.source_port,
                    packet_src_channel: packet.source_channel,
                    packet_dst_port: packet.destination_port,
                    packet_dst_channel: packet.destination_channel,
                    packet_channel_ordering: channel.ordering,
                    connection_id: channel.connection_hops[0].clone(),
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
