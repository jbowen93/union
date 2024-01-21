pragma solidity ^0.8.23;

import "@openzeppelin/utils/Strings.sol";
import "./ILightClient.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../02-client/IIBCClient.sol";

library IBCClientLib {
    event GeneratedClientIdentifier(string);

    error ErrClientTypeAlreadyExists();
    error ErrClientMustNotBeSelf();
    error ErrClientTypeNotFound();
    error ErrFailedToCreateClient();
    error ErrFailedToUpdateClient();
    error ErrClientNotFound();
}

/**
 * @dev IBCClient is a contract that implements [ICS-2](https://github.com/cosmos/ibc/tree/main/spec/core/ics-002-client-semantics).
 */
contract IBCClient is IBCStore, IIBCClient {
    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) external override {
        if (address(clientRegistry[clientType]) != address(0)) {
            revert IBCClientLib.ErrClientTypeAlreadyExists();
        }
        if (address(client) == address(this)) {
            revert IBCClientLib.ErrClientMustNotBeSelf();
        }
        clientRegistry[clientType] = address(client);
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(
        IBCMsgs.MsgCreateClient calldata msg_
    ) external override returns (string memory clientId) {
        address clientImpl = clientRegistry[msg_.clientType];
        if (clientImpl == address(0)) {
            revert IBCClientLib.ErrClientTypeNotFound();
        }
        clientId = generateClientIdentifier(msg_.clientType);
        clientTypes[clientId] = msg_.clientType;
        clientImpls[clientId] = clientImpl;
        (
            bytes32 clientStateCommitment,
            ConsensusStateUpdate memory update,
            bool ok
        ) = ILightClient(clientImpl).createClient(
                clientId,
                msg_.clientStateBytes,
                msg_.consensusStateBytes
            );
        if (!ok) {
            revert IBCClientLib.ErrFailedToCreateClient();
        }

        // update commitments
        commitments[
            keccak256(IBCCommitment.clientStatePath(clientId))
        ] = clientStateCommitment;
        commitments[
            IBCCommitment.consensusStateCommitmentKey(
                clientId,
                update.height.revision_number,
                update.height.revision_height
            )
        ] = update.consensusStateCommitment;

        emit IBCClientLib.GeneratedClientIdentifier(clientId);

        return clientId;
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(
        IBCMsgs.MsgUpdateClient calldata msg_
    ) external override {
        if (
            commitments[
                IBCCommitment.clientStateCommitmentKey(msg_.clientId)
            ] == bytes32(0)
        ) {
            revert IBCClientLib.ErrClientNotFound();
        }
        (
            bytes32 clientStateCommitment,
            ConsensusStateUpdate[] memory updates,
            bool ok
        ) = getClient(msg_.clientId).updateClient(
                msg_.clientId,
                msg_.clientMessage
            );
        if (!ok) {
            revert IBCClientLib.ErrFailedToUpdateClient();
        }

        // update commitments
        commitments[
            keccak256(IBCCommitment.clientStatePath(msg_.clientId))
        ] = clientStateCommitment;
        for (uint256 i = 0; i < updates.length; i++) {
            commitments[
                IBCCommitment.consensusStateCommitmentKey(
                    msg_.clientId,
                    updates[i].height.revision_number,
                    updates[i].height.revision_height
                )
            ] = updates[i].consensusStateCommitment;
        }
    }

    function generateClientIdentifier(
        string calldata clientType
    ) private returns (string memory) {
        string memory identifier = string(
            abi.encodePacked(
                clientType,
                "-",
                Strings.toString(nextClientSequence)
            )
        );
        nextClientSequence++;
        return identifier;
    }
}
