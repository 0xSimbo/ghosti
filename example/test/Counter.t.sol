    // SPDX-License-Identifier: UNLICENSED
    pragma solidity ^0.8.13;

    import {Test, console} from "forge-std/Test.sol";
    import {Counter} from "../_ghosti/counter/Counter.sol";

contract CounterTest is Test {
    Counter public counter;

    function setUp() public {
        counter = new Counter();
    }

    function testFuzz_addNumbers(uint256[20] memory numbers) public {
        for (uint256 i = 0; i < numbers.length; i++) {
            // example to prevent overflow
            uint256 num = bound(numbers[i], 1, 100);
            counter.add(num);
        }
        assertEq(counter.getGhostiSum(), sumOfArray(counter.getAllNumbers()));
    }

    function sumOfArray(uint256[] memory numbers) public pure returns (uint256) {
        uint256 sum = 0;
        for (uint256 i = 0; i < numbers.length; i++) {
            sum += numbers[i];
        }
        return sum;
    }
}
