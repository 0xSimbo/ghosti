// A trivial counter that uses GhostiBase.
pragma solidity ^0.8.0;

import "./GhostiBase.sol";

contract Counter is GhostiBase {
    function increment() public {
        getGhostiStorage().count++;
    }

    function get() public view returns (uint256) {
        // For demonstration only
        return getGhostiStorage().count;
    }
} 