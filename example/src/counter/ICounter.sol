// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

interface ICounter {
    function add(uint256 number) external;
    function getAllNumbers() external view returns (uint256[] memory);
}
