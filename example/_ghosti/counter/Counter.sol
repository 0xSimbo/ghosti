// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./GhostiBase.sol";
import "./ICounter.sol";

contract Counter is GhostiBase, ICounter {
    uint256[] public numbers;

    function add(uint256 number) public {
        numbers.push(number);
        getGhostiStorage().ghosti_numbersSum += number;
    }

    function getAllNumbers() public view returns (uint256[] memory) {
        return numbers;
    }
}
