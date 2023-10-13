pragma solidity ^0.8.21;

import "../../proto/ibc/core/client/v1/client.sol";

library IBCHeight {
    function toUint128(
        IbcCoreClientV1Height.Data memory self
    ) internal pure returns (uint128) {
        return
            (uint128(self.revision_number) << 64) |
            uint128(self.revision_height);
    }

    function isZero(
        IbcCoreClientV1Height.Data memory self
    ) internal pure returns (bool) {
        return self.revision_number == 0 && self.revision_height == 0;
    }

    function lt(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) internal pure returns (bool) {
        return
            self.revision_number < other.revision_number ||
            (self.revision_number == other.revision_number &&
                self.revision_height < other.revision_height);
    }

    function lte(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) internal pure returns (bool) {
        return
            self.revision_number < other.revision_number ||
            (self.revision_number == other.revision_number &&
                self.revision_height <= other.revision_height);
    }

    function eq(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) internal pure returns (bool) {
        return
            self.revision_number == other.revision_number &&
            self.revision_height == other.revision_height;
    }

    function gt(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) internal pure returns (bool) {
        return
            self.revision_number > other.revision_number ||
            (self.revision_number == other.revision_number &&
                self.revision_height > other.revision_height);
    }

    function gte(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) internal pure returns (bool) {
        return
            self.revision_number > other.revision_number ||
            (self.revision_number == other.revision_number &&
                self.revision_height >= other.revision_height);
    }
}
