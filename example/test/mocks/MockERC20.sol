// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract MockERC20 {
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    bool public transferShouldRevert;

    function mint(address to, uint256 amount) public {
        balanceOf[to] += amount;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        if (transferShouldRevert) {
            revert("MockERC20: transfer reverted");
        }
        require(balanceOf[msg.sender] >= amount, "MockERC20: insufficient balance");
        
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        require(!transferShouldRevert, "MockERC20: transfer reverted");
        require(balanceOf[from] >= amount, "MockERC20: insufficient balance");
        require(allowance[from][msg.sender] >= amount, "MockERC20: insufficient allowance");

        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function setTransferShouldRevert(bool shouldRevert) public {
        transferShouldRevert = shouldRevert;
    }
}
