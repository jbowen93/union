#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Reply, Response, SubMsgResult,
};
use prost::{Message, Name};
use protos::cosmwasm::wasm::v1::MsgIbcSendResponse;
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::InFlightPfmPacket,
    protocol::{TransferProtocol, IBC_SEND_ID},
};

use crate::{
    error::ContractError,
    protocol::{protocol_ordering, Ics20Protocol, ProtocolCommon, Ucs01Protocol},
    state::{ChannelInfo, PfmRefundPacketKey, CHANNEL_INFO, IN_FLIGHT_PFM_PACKETS},
};

fn to_response<T>(
    IbcReceiveResponse {
        acknowledgement,
        messages,
        attributes,
        events,
        ..
    }: IbcReceiveResponse<T>,
) -> Response<T> {
    let response = Response::<T>::new()
        .add_submessages(messages)
        .add_attributes(attributes)
        .add_events(events);

    if let Some(ack) = acknowledgement {
        response.set_data(ack)
    } else {
        response
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    _: Env,
    reply: Reply,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match (reply.id, reply.result) {
        // RECEIVE_REPLY_ID is associated with submessages emitted during handling of `ibc_packet_receive`
        (Ics20Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ics20Protocol::receive_error(err)))
        }
        (Ucs01Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ucs01Protocol::receive_error(err)))
        }
        // IBC_SEND_ID is associated with submessages emitted during handling of `send`, which is called via `execute_transfer`, which is used both in PFM and non-PFM contexts
        (IBC_SEND_ID, SubMsgResult::Ok(value)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Ok(Response::new());
            }

            let msg_response = value
                .msg_responses
                .iter()
                .find(|msg_response| msg_response.type_url == MsgIbcSendResponse::type_url())
                .expect("type url is correct and exists");

            let send_response =
                MsgIbcSendResponse::decode(msg_response.value.as_slice()).expect("is type url");

            let in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            let refund_packet_key = PfmRefundPacketKey {
                channel_id: in_flight_packet.forward_src_channel_id.clone(),
                port_id: in_flight_packet.forward_src_port_id.clone(),
                sequence: send_response.sequence,
            };

            IN_FLIGHT_PFM_PACKETS
                .save(deps.storage, refund_packet_key.clone(), &in_flight_packet)
                .expect("infallible update");

            Ok(
                Response::new()
                    .add_event(in_flight_packet.create_hop_event(send_response.sequence)),
            )
        }
        (IBC_SEND_ID, SubMsgResult::Err(err)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Err(ContractError::PfmSendPacketError { err });
            }

            // decode the payload to figure out the source channel
            let in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            match &*in_flight_packet.origin_protocol_version {
                Ucs01Protocol::VERSION => Ok(to_response(Ucs01Protocol::receive_error(err))),
                Ics20Protocol::VERSION => Ok(to_response(Ics20Protocol::receive_error(err))),
                // in_flight_packet.origin_protocol_version is only ever set by us, so if it is set incorrectly then it is a bug
                version => unreachable!("unknown protocol version: {version}"),
            }
        }
        (_, result) => Err(ContractError::UnknownReply {
            id: reply.id,
            variant: result,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        endpoint: channel.endpoint,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
        protocol_version: channel.version,
    };
    CHANNEL_INFO.save(deps.storage, &info.endpoint.channel_id, &info)?;
    Ok(IbcBasicResponse::default())
}

pub(crate) fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    let channel_ordering =
        protocol_ordering(&channel.version).ok_or(ContractError::UnknownProtocol {
            channel_id: channel.endpoint.channel_id.clone(),
            protocol_version: channel.version.clone(),
        })?;
    if let Some(version) = counterparty_version {
        if protocol_ordering(version).is_none() {
            return Err(ContractError::UnknownProtocol {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: version.to_string(),
            });
        }
        if version != channel.version {
            return Err(ContractError::ProtocolMismatch {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: channel.version.clone(),
                counterparty_protocol_version: version.to_string(),
            });
        }
    }
    if channel.order != channel_ordering {
        return Err(ContractError::InvalidChannelOrdering {
            expected: channel_ordering,
            actual: channel.order.clone(),
        });
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Err(ContractError::Unauthorized)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.dest.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => Ok(Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .receive(msg.packet)),
        Ucs01Protocol::VERSION => Ok(Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .receive(msg.packet)),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.original_packet.src.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer.clone(),
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_ack(msg),
        Ucs01Protocol::VERSION => Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_ack(msg),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.original_packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.src.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_timeout(msg.packet),
        Ucs01Protocol::VERSION => Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_timeout(msg.packet),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{IbcChannel, IbcEndpoint};
    use ucs01_relay_api::protocol::TransferProtocol;

    use super::enforce_order_and_version;
    use crate::{
        error::ContractError,
        protocol::{Ics20Protocol, Ucs01Protocol},
    };

    #[test]
    fn enforce_channel_version_ucs01() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        )
        .unwrap();
    }

    #[test]
    fn enforce_channel_version_ics20() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ics20Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        )
        .unwrap()
    }

    #[test]
    fn enforce_channel_wrong_version() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = "ucs01-0999";
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        match enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        ) {
            Err(ContractError::UnknownProtocol {
                channel_id: unknown_channel_id,
                protocol_version: unknown_protocol_version,
            }) => {
                assert_eq!(unknown_channel_id, channel_id);
                assert_eq!(unknown_protocol_version, protocol_version);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn enforce_channel_counterparty_wrong_version() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        let counterparty_protocol_version = "ucs01-0999";
        match enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            Some(counterparty_protocol_version),
        ) {
            Err(ContractError::UnknownProtocol {
                channel_id: unknown_channel_id,
                protocol_version: unknown_protocol_version,
            }) => {
                assert_eq!(unknown_channel_id, channel_id);
                assert_eq!(unknown_protocol_version, counterparty_protocol_version);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn enforce_channel_protocol_mismatch() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        let counterparty_protocol_version = Ics20Protocol::VERSION;
        let mismatch = enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            Some(counterparty_protocol_version),
        );
        match mismatch {
            Err(ContractError::ProtocolMismatch {
                channel_id: mismatch_channel_id,
                protocol_version: mismatch_protocol_version,
                counterparty_protocol_version: mismatch_counterparty_protocol_version,
            }) => {
                assert_eq!(mismatch_channel_id, channel_id);
                assert_eq!(mismatch_protocol_version, protocol_version);
                assert_eq!(
                    mismatch_counterparty_protocol_version,
                    counterparty_protocol_version
                );
            }
            _ => panic!(),
        }
    }
}
