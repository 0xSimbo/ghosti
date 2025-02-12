// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./IERC20.sol";

contract Vault {
    IERC20 public immutable token;
    mapping(address => uint256) public balances;
    uint256 public constant FEE = 10; // 0.1%

    constructor(address _token) {
        token = IERC20(_token);
    }

    /// @notice Deposit tokens into the vault
    function deposit(uint256 amount) external {
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        uint256 fee = (amount * FEE) / 10000;
        uint256 afterFee = amount - fee;
        balances[msg.sender] += afterFee;

        // Track for testing
    }

    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;

        // Track failed transfers for security analysis
        if (!token.transfer(msg.sender, amount)) {
            balances[msg.sender] += amount; // Revert balance
            revert("Transfer failed");
        }

        // Update test tracking
    }
}
