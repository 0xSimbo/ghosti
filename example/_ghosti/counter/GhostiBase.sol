// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct GhostiStorage {
    uint256 ghosti_numbersSum;
}
// You can add more storage fields here...

contract GhostiBase {
    bytes32 constant GHOSTI_STORAGE_SLOT = keccak256("ghosti.counter.storage");

    function getGhostiStorage() internal pure returns (GhostiStorage storage gs) {
        bytes32 slot = GHOSTI_STORAGE_SLOT;
        assembly {
            gs.slot := slot
        }
    }

    function getGhostiSum() public view returns (uint256) {
        return getGhostiStorage().ghosti_numbersSum;
    }
}
