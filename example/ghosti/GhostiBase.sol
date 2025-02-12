// A base contract providing GhostiStorage and getGhostiStorage().
pragma solidity ^0.8.0;

struct GhostiStorage {
    uint256 count;
    // You can add more storage fields here...
}

contract GhostiBase {
    function getGhostiStorage() internal pure returns (GhostiStorage storage gs) {
        // This pattern is similar to diamond storage pointing
        assembly {
            gs.slot := 123
        }
    }
} 