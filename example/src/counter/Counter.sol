// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./ICounter.sol";

contract Counter is ICounter {
    uint256[] public numbers;

    function add(uint256 number) public {
        numbers.push(number);
    }

    function getAllNumbers() public view returns (uint256[] memory) {
        return numbers;
    }
}
