// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "@/_ghosti/vault/Vault.sol";
import "@/_ghosti/vault/IERC20.sol";
import {MockERC20} from "./mocks/MockERC20.sol";

contract VaultTest is Test {
    Vault vault;
    MockERC20 token;
    address alice = address(0x1);
    address bob = address(0x2);
    uint256 constant INITIAL_BALANCE = 1000e18;
    uint256 constant FEE_DENOMINATOR = 10000;

    event Deposit(address indexed user, uint256 amount, uint256 fee);
    event Withdrawal(address indexed user, uint256 amount);

    function setUp() public {
        // Deploy contracts
        token = new MockERC20();
        vault = new Vault(address(token));

        // Setup test users
        token.mint(alice, INITIAL_BALANCE);
        token.mint(bob, INITIAL_BALANCE);
        
        vm.label(alice, "Alice");
        vm.label(bob, "Bob");
    }

    function testDeposit() public {
        uint256 depositAmount = 100e18;
        uint256 expectedFee = (depositAmount * vault.FEE()) / FEE_DENOMINATOR;
        uint256 expectedBalance = depositAmount - expectedFee;

        // Approve and deposit as Alice
        vm.startPrank(alice);
        token.approve(address(vault), depositAmount);
        vault.deposit(depositAmount);
        vm.stopPrank();

        // Check balances
        assertEq(vault.balances(alice), expectedBalance, "Vault balance incorrect");
        assertEq(token.balanceOf(address(vault)), depositAmount, "Token balance incorrect");
        assertEq(token.balanceOf(alice), INITIAL_BALANCE - depositAmount, "User balance incorrect");
        
        // Check test tracking
        assertEq(vault.getTotalDeposits(), depositAmount, "Total deposits incorrect");
        assertEq(vault.getUserDeposit(alice), expectedBalance, "User deposit tracking incorrect");
    }

    function testWithdraw() public {
        // Setup: First deposit
        uint256 depositAmount = 100e18;
        vm.startPrank(alice);
        token.approve(address(vault), depositAmount);
        vault.deposit(depositAmount);

        // Calculate actual balance after fees
        uint256 fee = (depositAmount * vault.FEE()) / FEE_DENOMINATOR;
        uint256 withdrawAmount = depositAmount - fee;

        // Now withdraw
        vault.withdraw(withdrawAmount);
        vm.stopPrank();

        // Check balances
        assertEq(vault.balances(alice), 0, "Vault balance should be 0");
        assertEq(
            token.balanceOf(alice), 
            INITIAL_BALANCE - fee, 
            "User should receive original amount minus fees"
        );
    }

    function test_WithdrawTooMuch() public {
        uint256 depositAmount = 100e18;
        vm.startPrank(alice);
        token.approve(address(vault), depositAmount);
        vault.deposit(depositAmount);

        // Try to withdraw more than deposited
        vm.expectRevert("Insufficient balance");
        vault.withdraw(depositAmount + 1);
        vm.stopPrank();
    }

    function testMultipleUsersDeposit() public {
        uint256 aliceAmount = 100e18;
        uint256 bobAmount = 150e18;

        // Alice deposits
        vm.startPrank(alice);
        token.approve(address(vault), aliceAmount);
        vault.deposit(aliceAmount);
        vm.stopPrank();

        // Bob deposits
        vm.startPrank(bob);
        token.approve(address(vault), bobAmount);
        vault.deposit(bobAmount);
        vm.stopPrank();

        // Calculate expected balances after fees
        uint256 aliceFee = (aliceAmount * vault.FEE()) / FEE_DENOMINATOR;
        uint256 bobFee = (bobAmount * vault.FEE()) / FEE_DENOMINATOR;

        // Check individual balances
        assertEq(vault.balances(alice), aliceAmount - aliceFee, "Alice's balance incorrect");
        assertEq(vault.balances(bob), bobAmount - bobFee, "Bob's balance incorrect");

        // Check total deposits
        assertEq(
            vault.getTotalDeposits(),
            aliceAmount + bobAmount,
            "Total deposits incorrect"
        );
    }


    function testInvariants() public {
        // Setup some activity
        uint256 aliceDeposit = 100e18;
        uint256 bobDeposit = 150e18;

        // Alice deposits
        vm.startPrank(alice);
        token.approve(address(vault), aliceDeposit);
        vault.deposit(aliceDeposit);
        uint256 aliceFee = (aliceDeposit * vault.FEE()) / FEE_DENOMINATOR;
        uint256 aliceBalance = aliceDeposit - aliceFee;
        
        // Withdraw part of balance
        uint256 withdrawAmount = 40e18;
        vault.withdraw(withdrawAmount);
        aliceBalance -= withdrawAmount;  // Update Alice's expected balance
        vm.stopPrank();

        // Bob deposits
        vm.startPrank(bob);
        token.approve(address(vault), bobDeposit);
        vault.deposit(bobDeposit);
        uint256 bobFee = (bobDeposit * vault.FEE()) / FEE_DENOMINATOR;
        uint256 bobBalance = bobDeposit - bobFee;
        vm.stopPrank();

        // Check invariants
        assertEq(
            vault.balances(alice) + vault.balances(bob),
            aliceBalance + bobBalance,
            "Sum of balances mismatch"
        );

        // Check individual balances match tracking
        assertEq(
            vault.getUserDeposit(alice),
            vault.balances(alice),
            "Alice's deposit tracking mismatch"
        );
        assertEq(
            vault.getUserDeposit(bob),
            vault.balances(bob),
            "Bob's deposit tracking mismatch"
        );

        // Verify no failed withdrawals
        assertEq(
            vault.failedWithdrawals(),
            0,
            "Should have no failed withdrawals"
        );
    }
}