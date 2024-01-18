pragma solidity ^0.8.23;

import "@openzeppelin/contracts/utils/Strings.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/ModuleManager.sol";

/**
 * @dev IBCPacket is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
contract IBCPacket is IBCStore, IIBCPacket, ModuleManager {
    using IBCHeight for IbcCoreClientV1Height.Data;

    /* Packet handlers */

    /**
     * @dev sendPacket is called by a module in order to send an IBC packet on a channel.
     * The packet sequence generated for the packet to be sent is returned. An error
     * is returned if one occurs.
     */
    function sendPacket(
        string calldata sourcePort,
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external returns (uint64) {
        IbcCoreChannelV1Channel.Data storage channel = channels[sourcePort][
            sourceChannel
        ];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            "sendPacket: channel state must be OPEN"
        );

        {
            IbcCoreConnectionV1ConnectionEnd.Data
                storage connection = connections[channel.connection_hops[0]];
            ILightClient client = getClient(connection.client_id);

            (
                IbcCoreClientV1Height.Data memory latestHeight,
                bool found
            ) = client.getLatestHeight(connection.client_id);
            require(found, "sendPacket: latest height not found");
            require(
                timeoutHeight.isZero() || latestHeight.lt(timeoutHeight),
                "sendPacket: receiving chain block height >= packet timeout height"
            );
            uint64 latestTimestamp;
            (latestTimestamp, found) = client.getTimestampAtHeight(
                connection.client_id,
                latestHeight
            );
            require(found, "sendPacket: timestamp not found");
            require(
                timeoutTimestamp == 0 || latestTimestamp < timeoutTimestamp,
                "sendPacket: receiving chain block timestamp >= packet timeout timestamp"
            );
        }

        uint64 packetSequence = nextSequenceSends[sourcePort][sourceChannel];
        nextSequenceSends[sourcePort][sourceChannel] = packetSequence + 1;
        commitments[
            IBCCommitment.packetCommitmentKey(
                sourcePort,
                sourceChannel,
                packetSequence
            )
        ] = keccak256(
            abi.encodePacked(
                sha256(
                    abi.encodePacked(
                        timeoutTimestamp,
                        timeoutHeight.revision_number,
                        timeoutHeight.revision_height,
                        sha256(data)
                    )
                )
            )
        );
        return packetSequence;
    }

    /**
     * @dev recvPacket is called by a module in order to receive & process an IBC packet
     * sent on the corresponding channel end on the counterparty chain.
     */
    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[
            msg_.packet.destination_port
        ][msg_.packet.destination_channel];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            "recvPacket: channel state must be OPEN"
        );

        require(
            hashString(msg_.packet.source_port) ==
                hashString(channel.counterparty.port_id),
            "recvPacket: packet source port doesn't match the counterparty's port"
        );
        require(
            hashString(msg_.packet.source_channel) ==
                hashString(channel.counterparty.channel_id),
            "recvPacket: packet source channel doesn't match the counterparty's channel"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "recvPacket: connection state is not OPEN"
        );

        require(
            msg_.packet.timeout_height.revision_height == 0 ||
                (block.number < msg_.packet.timeout_height.revision_height),
            "recvPacket: block height >= packet timeout height"
        );
        require(
            msg_.packet.timeout_timestamp == 0 ||
                (block.timestamp < msg_.packet.timeout_timestamp),
            "recvPacket: block timestamp >= packet timeout timestamp"
        );

        require(
            verifyCommitment(
                connection,
                msg_.proofHeight,
                msg_.proof,
                IBCCommitment.packetCommitmentPath(
                    msg_.packet.source_port,
                    msg_.packet.source_channel,
                    msg_.packet.sequence
                ),
                abi.encodePacked(
                    sha256(
                        abi.encodePacked(
                            msg_.packet.timeout_timestamp,
                            msg_.packet.timeout_height.revision_number,
                            msg_.packet.timeout_height.revision_height,
                            sha256(msg_.packet.data)
                        )
                    )
                )
            ),
            "recvPacket: failed to verify packet commitment"
        );

        if (
            channel.ordering ==
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED
        ) {
            require(
                packetReceipts[msg_.packet.destination_port][
                    msg_.packet.destination_channel
                ][msg_.packet.sequence] == 0,
                "recvPacket: packet sequence already has been received"
            );
            packetReceipts[msg_.packet.destination_port][
                msg_.packet.destination_channel
            ][msg_.packet.sequence] = 1;
        } else if (
            channel.ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED
        ) {
            require(
                nextSequenceRecvs[msg_.packet.destination_port][
                    msg_.packet.destination_channel
                ] == msg_.packet.sequence,
                "recvPacket: packet sequence != next receive sequence"
            );
            nextSequenceRecvs[msg_.packet.destination_port][
                msg_.packet.destination_channel
            ]++;
        } else {
            revert("recvPacket: unknown ordering type");
        }
    }

    /**
     * @dev writeAcknowledgement writes the packet execution acknowledgement to the state,
     * which will be verified by the counterparty chain using AcknowledgePacket.
     */
    function writeAcknowledgement(
        string calldata destinationPortId,
        string calldata destinationChannel,
        uint64 sequence,
        bytes calldata acknowledgement
    ) external {
        require(
            acknowledgement.length > 0,
            "writeAcknowlegement: acknowledgement cannot be empty"
        );

        IbcCoreChannelV1Channel.Data storage channel = channels[
            destinationPortId
        ][destinationChannel];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            "writeAcknowlegement: channel state must be OPEN"
        );

        bytes32 ackCommitmentKey = IBCCommitment
            .packetAcknowledgementCommitmentKey(
                destinationPortId,
                destinationChannel,
                sequence
            );
        bytes32 ackCommitment = commitments[ackCommitmentKey];
        require(
            ackCommitment == bytes32(0),
            "writeAcknowlegement: acknowledgement for packet already exists"
        );
        commitments[ackCommitmentKey] = keccak256(
            abi.encodePacked(sha256(acknowledgement))
        );
    }

    /**
     * @dev AcknowledgePacket is called by a module to process the acknowledgement of a
     * packet previously sent by the calling module on a channel to a counterparty
     * module on the counterparty chain. Its intended usage is within the ante
     * handler. AcknowledgePacket will clean up the packet commitment,
     * which is no longer necessary since the packet has been received and acted upon.
     * It will also increment NextSequenceAck in case of ORDERED channels.
     */
    function acknowledgePacket(
        IBCMsgs.MsgPacketAcknowledgement calldata msg_
    ) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[
            msg_.packet.source_port
        ][msg_.packet.source_channel];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            "acknowledgePacket: channel state must be OPEN"
        );

        require(
            hashString(msg_.packet.destination_port) ==
                hashString(channel.counterparty.port_id),
            "acknowledgePacket: packet destination port doesn't match the counterparty's port"
        );
        require(
            hashString(msg_.packet.destination_channel) ==
                hashString(channel.counterparty.channel_id),
            "acknowledgePacket: packet destination channel doesn't match the counterparty's channel"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "acknowledgePacket: connection state is not OPEN"
        );

        bytes32 packetCommitmentKey = IBCCommitment.packetCommitmentKey(
            msg_.packet.source_port,
            msg_.packet.source_channel,
            msg_.packet.sequence
        );
        bytes32 packetCommitment = commitments[packetCommitmentKey];
        require(
            packetCommitment != bytes32(0),
            "acknowledgePacket: packet commitment not found"
        );
        require(
            packetCommitment ==
                keccak256(
                    abi.encodePacked(
                        sha256(
                            abi.encodePacked(
                                msg_.packet.timeout_timestamp,
                                msg_.packet.timeout_height.revision_number,
                                msg_.packet.timeout_height.revision_height,
                                sha256(msg_.packet.data)
                            )
                        )
                    )
                ),
            "acknowledgePacket: commitment bytes are not equal"
        );

        require(
            verifyCommitment(
                connection,
                msg_.proofHeight,
                msg_.proof,
                IBCCommitment.packetAcknowledgementCommitmentPath(
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence
                ),
                abi.encodePacked(sha256(msg_.acknowledgement))
            ),
            "acknowledgePacket: failed to verify packet acknowledgement commitment"
        );

        if (
            channel.ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED
        ) {
            require(
                msg_.packet.sequence ==
                    nextSequenceAcks[msg_.packet.source_port][
                        msg_.packet.source_channel
                    ],
                "acknowledgePacket: packet sequence != next ack sequence"
            );
            nextSequenceAcks[msg_.packet.source_port][
                msg_.packet.source_channel
            ]++;
        }

        delete commitments[packetCommitmentKey];
    }

    function hashString(string memory s) private pure returns (bytes32) {
        return keccak256(abi.encodePacked(s));
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata msg_) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[
            msg_.packet.source_port
        ][msg_.packet.source_channel];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            "timeoutPacket: channel state must be OPEN"
        );

        require(
            hashString(msg_.packet.destination_port) ==
                hashString(channel.counterparty.port_id),
            "timeoutPacket: packet destination port doesn't match the counterparty's port"
        );
        require(
            hashString(msg_.packet.destination_channel) ==
                hashString(channel.counterparty.channel_id),
            "timeoutPacket: packet destination channel doesn't match the counterparty's channel"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "timeoutPacket: connection state is not OPEN"
        );

        bytes32 packetCommitmentKey = IBCCommitment.packetCommitmentKey(
            msg_.packet.source_port,
            msg_.packet.source_channel,
            msg_.packet.sequence
        );
        bytes32 packetCommitment = commitments[packetCommitmentKey];
        require(
            packetCommitment != bytes32(0),
            "timeoutPacket: packet commitment not found"
        );
        require(
            packetCommitment ==
                keccak256(
                    abi.encodePacked(
                        sha256(
                            abi.encodePacked(
                                msg_.packet.timeout_timestamp,
                                msg_.packet.timeout_height.revision_number,
                                msg_.packet.timeout_height.revision_height,
                                sha256(msg_.packet.data)
                            )
                        )
                    )
                ),
            "timeoutPacket: commitment bytes are not equal"
        );

        ILightClient client = getClient(connection.client_id);
        (uint64 proofTimestamp, bool found) = client.getTimestampAtHeight(
            connection.client_id,
            msg_.proofHeight
        );
        require(found, "timeoutPacket: timestamp not found");

        if (
            msg_.packet.timeout_timestamp == 0 &&
            msg_.packet.timeout_height.isZero()
        ) {
            revert("timeoutPacket: packet has no timestamp/height timeout");
        }
        if (msg_.packet.timeout_timestamp > 0) {
            require(
                msg_.packet.timeout_timestamp < proofTimestamp,
                "timeoutPacket: timestamp timeout not reached for the given proof height"
            );
        }
        if (!msg_.packet.timeout_height.isZero()) {
            require(
                msg_.packet.timeout_height.lt(msg_.proofHeight),
                "timeoutPacket: height timeout not reached for the given proof height"
            );
        }

        bool isOrdered = channel.ordering ==
            IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED;
        bool isUnordered = channel.ordering ==
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED;
        if (isOrdered) {
            require(
                msg_.nextSequenceRecv > msg_.packet.sequence,
                "timeoutPacket: nextSequenceRecv must be > packet to timeout"
            );
            require(
                verifyCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.nextSequenceRecvCommitmentPath(
                        msg_.packet.destination_port,
                        msg_.packet.destination_channel
                    ),
                    abi.encodePacked(msg_.nextSequenceRecv)
                ),
                "timeoutPacket: failed to verify packet timeout next recv proof"
            );
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        } else if (isUnordered) {
            require(
                verifyAbsentCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.packetReceiptCommitmentPath(
                        msg_.packet.destination_port,
                        msg_.packet.destination_channel,
                        msg_.packet.sequence
                    )
                ),
                "timeoutPacket: failed to verify packet timeout absence proof"
            );
        } else {
            revert("timeoutPacket: unknown ordering type");
        }

        delete commitments[packetCommitmentKey];
    }

    function verifyCommitment(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        bytes memory path,
        bytes memory commitment
    ) private returns (bool) {
        return
            getClient(connection.client_id).verifyMembership(
                connection.client_id,
                height,
                connection.delay_period,
                0,
                proof,
                connection.counterparty.prefix.key_prefix,
                path,
                commitment
            );
    }

    function verifyAbsentCommitment(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        bytes memory path
    ) private returns (bool) {
        return
            getClient(connection.client_id).verifyNonMembership(
                connection.client_id,
                height,
                connection.delay_period,
                0,
                proof,
                connection.counterparty.prefix.key_prefix,
                path
            );
    }
}
