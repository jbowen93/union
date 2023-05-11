// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "../../proto/ibc/core/client/v1/client.sol";
import "../02-client/ILightClient.sol";
import "../24-host/IBCStore.sol";
import "../05-port/ModuleManager.sol";
import "../24-host/IBCCommitment.sol";

abstract contract IBCQuerier is IBCStore {
    function getClientState(string calldata clientId) external view returns (bytes memory, bool) {
        return getClient(clientId).getClientState(clientId);
    }

    function getConsensusState(string calldata clientId, IbcCoreClientV1Height.Data calldata height)
        external
        view
        returns (bytes memory consensusStateBytes, bool)
    {
        return getClient(clientId).getConsensusState(clientId, height);
    }

    function getConnection(string calldata connectionId) external view returns (IbcCoreConnectionV1ConnectionEnd.Data memory, bool) {
        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[connectionId];
        return (connection, connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_UNINITIALIZED_UNSPECIFIED);
    }

    function getChannel(string calldata portId, string calldata channelId)
        external
        view
        returns (IbcCoreChannelV1Channel.Data memory, bool)
    {
        IbcCoreChannelV1Channel.Data storage channel = channels[portId][channelId];
        return (channel, channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_UNINITIALIZED_UNSPECIFIED);
    }

    function getHashedPacketCommitment(string calldata portId, string calldata channelId, uint64 sequence)
        external
        view
        returns (bytes32, bool)
    {
        bytes32 commitment = commitments[keccak256(IBCCommitment.packetCommitmentPath(portId, channelId, sequence))];
        return (commitment, commitment != bytes32(0));
    }

    function getHashedPacketAcknowledgementCommitment(
        string calldata portId,
        string calldata channelId,
        uint64 sequence
    ) external view returns (bytes32, bool) {
        bytes32 commitment =
            commitments[keccak256(IBCCommitment.packetAcknowledgementCommitmentPath(portId, channelId, sequence))];
        return (commitment, commitment != bytes32(0));
    }

    function hasPacketReceipt(string calldata portId, string calldata channelId, uint64 sequence)
        external
        view
        returns (bool)
    {
        return packetReceipts[portId][channelId][sequence] == 1;
    }

    function getNextSequenceSend(string calldata portId, string calldata channelId) external view returns (uint64) {
        return nextSequenceSends[portId][channelId];
    }

    function getExpectedTimePerBlock() external view returns (uint64) {
        return expectedTimePerBlock;
    }
}
