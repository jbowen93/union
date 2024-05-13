pragma solidity ^0.8.23;

import "../../proto/ibc/core/connection/v1/connection.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../02-client/ILightClient.sol";

library IBCStoreLib {
    error ErrClientNotFound();
}

abstract contract IBCStore {
    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) public commitments;

    // Store
    mapping(string => address) public clientRegistry;
    mapping(string => string) public clientTypes;
    mapping(string => address) public clientImpls;
    mapping(string => IbcCoreConnectionV1ConnectionEnd.Data) public connections;
    mapping(string => mapping(string => IbcCoreChannelV1Channel.Data)) public
        channels;
    mapping(string => address) public capabilities;

    // Sequences for identifier
    bytes32 public constant nextClientSequencePath =
        keccak256("nextClientSequence");
    bytes32 public constant nextConnectionSequencePath =
        keccak256("nextConnectionSequence");
    bytes32 public constant nextChannelSequencePath =
        keccak256("nextChannelSequence");

    string public constant COMMITMENT_PREFIX = "ibc";

    // Storage accessors
    function getClient(string memory clientId)
        public
        view
        returns (ILightClient)
    {
        address clientImpl = clientImpls[clientId];
        if (clientImpl == address(0)) {
            revert IBCStoreLib.ErrClientNotFound();
        }
        return ILightClient(clientImpl);
    }

    function getConnection(string calldata connectionId)
        external
        view
        returns (IbcCoreConnectionV1ConnectionEnd.Data memory)
    {
        return connections[connectionId];
    }

    function getChannel(
        string calldata portId,
        string calldata channelId
    ) external view returns (IbcCoreChannelV1Channel.Data memory) {
        return channels[portId][channelId];
    }
}
