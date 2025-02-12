// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct GhostiStorage {
    // Vault-specific storage for testing
    uint256 totalDeposits;
    mapping(address => uint256) userDeposits;
    uint256 failedWithdrawals;
    uint256 totalFeesCollected;
}

contract GhostiBase {
    bytes32 constant GHOSTI_STORAGE_SLOT = keccak256("ghosti.vault.storage");

    function getGhostiStorage() internal pure returns (GhostiStorage storage gs) {
        bytes32 slot = GHOSTI_STORAGE_SLOT;
        assembly {
            gs.slot := slot
        }
    }

    // Public getters for testing
    function getTotalDeposits() public view returns (uint256) {
        return getGhostiStorage().totalDeposits;
    }

    function getUserDeposit(address user) public view returns (uint256) {
        return getGhostiStorage().userDeposits[user];
    }

    function totalFeesCollected() public view returns (uint256) {
        return getGhostiStorage().totalFeesCollected;
    }

    function failedWithdrawals() public view returns (uint256) {
        return getGhostiStorage().failedWithdrawals;
    }
}
