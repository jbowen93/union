pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "@openzeppelin/access/Ownable.sol";
import "./25-handler/IBCHandler.sol";

/**
 * @dev OwnableIBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract OwnableIBCHandler is IBCHandler, Ownable {
    using LibString for *;

    /**
     * @dev The arguments of constructor must satisfy the followings:
     * @param ibcClient is the address of a contract that implements `IIBCClient`.
     * @param ibcConnection is the address of a contract that implements `IIBCConnectionHandshake`.
     * @param ibcChannel is the address of a contract that implements `IIBCChannelHandshake`.
     * @param ibcPacket is the address of a contract that implements `IIBCPacket`.
     */
    constructor(
        address ibcClient,
        address ibcConnection,
        address ibcChannel,
        address ibcPacket
    )
        IBCHandler(ibcClient, ibcConnection, ibcChannel, ibcPacket)
        Ownable(_msgSender())
    {}

    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) public override onlyOwner {
        super.registerClient(clientType, client);
    }
}
