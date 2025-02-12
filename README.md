# Ghosti CLI

> Because sometimes you need testing superpowers inside your contracts.

## Overview

**Ghosti** is a command-line tool that lets you write powerful inline testing logic in your Solidity contracts, then magically remove it for production. While tools like Forge give you great testing capabilities from the outside, Ghosti lets you instrument your contracts from the inside - and then vanish that instrumentation when you deploy.

## Why Ghosti?

Traditional Solidity testing faces several challenges:
1. **Limited Internal Visibility**: Test frameworks can only see public state
2. **Complex Setup**: Writing invariant tests often requires elaborate harnesses
3. **Indirect State Manipulation**: Changing internal state for testing is cumbersome
4. **No Direct Tracking**: Monitoring internal behavior requires events or external tracking

Ghosti solves these by letting you:
1. **Add Internal Instrumentation**: Track any state you want during testing
2. **Write Self-Documenting Tests**: Testing logic lives with the contract
3. **Maintain Production Cleanliness**: All test code vanishes in production builds
4. **Keep Gas Efficiency**: No overhead in deployed contracts

## Examples

### Basic Example: Counter with Sum Tracking
```solidity
// _ghosti/counter/Counter.sol
contract Counter is GhostiBase {
    uint256[] public numbers;

    function add(uint256 number) public {
        numbers.push(number);
        // Track sum for testing - removed in production
        getGhostiStorage().ghosti_numbersSum += number;
    }

    function getAllNumbers() public view returns (uint256[] memory) {
        return numbers;
    }
}
```

GhostiBase looks like this
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct GhostiStorage {
    uint256 ghosti_numbersSum;
}
// You can add more storage fields here...

contract GhostiBase {
    bytes32 constant GHOSTI_STORAGE_SLOT = keccak256("ghosti.counter.storage");

    function getGhostiStorage() internal pure returns (GhostiStorage storage gs) {
        bytes32 slot = GHOSTI_STORAGE_SLOT;
        assembly {
            gs.slot := slot
        }
    }

    function getGhostiSum() public view returns (uint256) {
        return getGhostiStorage().ghosti_numbersSum;
    }
}
```

After `ghosti build`, becomes:
```solidity
// src/counter/Counter.sol
contract Counter {
    uint256[] public numbers;

    function add(uint256 number) public {
        numbers.push(number);
    }

    function getAllNumbers() public view returns (uint256[] memory) {
        return numbers;
    }
}
```

Then in your tests, you can interact with the contract's storage directly:

```solidity
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
``` 
### Advanced Example: Vault with Comprehensive Testing State
```solidity
// _ghosti/vault/Vault.sol
contract Vault is GhostiBase {
    IERC20 public immutable token;
    mapping(address => uint256) public balances;
    uint256 public constant FEE = 10; // 0.1%

    function deposit(uint256 amount) external {
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        uint256 fee = (amount * FEE) / 10000;
        uint256 afterFee = amount - fee;
        balances[msg.sender] += afterFee;

        // Track critical invariants for testing
        getGhostiStorage().totalDeposits += amount;
        getGhostiStorage().userDeposits[msg.sender] += afterFee;
        getGhostiStorage().totalFeesCollected += fee;
    }

    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;

        // Track failed transfers for security analysis
        if (!token.transfer(msg.sender, amount)) {
            getGhostiStorage().failedWithdrawals += 1;
            balances[msg.sender] += amount;
            revert("Transfer failed");
        }

        getGhostiStorage().userDeposits[msg.sender] -= amount;
    }
}
```

This lets you write powerful invariant tests:
```solidity
function testInvariants() public {
    // ... setup ...

    // Check total deposits match balances plus fees
    assertEq(
        vault.getTotalDeposits() - vault.totalFeesCollected(),
        vault.balances(alice) + vault.balances(bob),
        "Total deposits minus fees should equal sum of balances"
    );

    // Verify no failed withdrawals (security check)
    assertEq(
        vault.failedWithdrawals(),
        0,
        "Should have no failed withdrawals"
    );
}
```

But in production, you get clean, efficient code:
```solidity
// src/vault/Vault.sol
contract Vault {
    IERC20 public immutable token;
    mapping(address => uint256) public balances;
    uint256 public constant FEE = 10;

    function deposit(uint256 amount) external {
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        uint256 fee = (amount * FEE) / 10000;
        uint256 afterFee = amount - fee;
        balances[msg.sender] += afterFee;
    }

    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        require(token.transfer(msg.sender, amount), "Transfer failed");
    }
}
```

## Comparison with Other Tools

| Feature | Ghosti | Traditional Testing | Hardhat/Forge |
|---------|--------|-------------------|----------------|
| Internal State Access | ✅ Direct | ❌ Limited | ❌ External Only |
| Production Impact | ✅ None | ✅ None | ✅ None |
| Test Code Location | ✅ In Contract | ❌ Separate Files | ❌ Separate Files |
| State Tracking | ✅ Automatic | ❌ Manual Events | ❌ Manual Events |
| Learning Curve | ✅ Low | ✅ Low | ❌ Medium |

## Installation

```bash
cargo install ghosti
```

## Usage

```bash
ghosti init
```
init Initialize a new Ghosti project or scaffold missing components



```bash
ghosti build
```
Build (transform) the project, removing Ghosti references and copying to output


```bash
ghosti help
```
Help display help information




### 1. `ghosti init`
- **Usage**: `ghosti init --config <path_to_toml>`
- **Default**: `ghosti init` (looks for `ghosti.toml` in current dir)
- Creates a `_ghosti` folder if one doesn't exist
- Creates a `GhostiBase.sol` with some boilerplate storage code if none is found
- Creates a `ghosti.toml` if none is found

### 2. `ghosti build`
- **Usage**: `ghosti build --config <path_to_toml>`
- **Default**: `ghosti build` (uses `ghosti.toml` found in current dir)
- Scans `_ghosti` folder (or `ghosti_folder` from config)
- Removes references to `GhostiBase.sol` and inline testing storage
- Copies the cleaned contracts into `src` (or your chosen `output_folder`)


This will transform all `.sol` files in your `_ghosti` folder, removing:
- `import` statements referencing `GhostiBase.sol`
- Inheritance from `GhostiBase`
- The `getGhostiStorage()` calls
- Any ephemeral test logic you configure

### 3. `ghosti help`
- **Usage**: `ghosti help`
- Prints a short description of the CLI and usage details.

## Configuration: `ghosti.toml`

Below is a typical `ghosti.toml`:

```toml
ghosti_folder = "ghosti" # Where your ghosti (test) contracts live
output_folder = "src" # Where final production code will go

[[import_aliases]]
import_prefix = "@/ghosti/"
replace_with = "@/src/"
```



- **`ghosti_folder`**: The directory for dev/test contracts. Defaults to `"_ghosti"`.
- **`output_folder`**: Where the cleaned code is placed. Defaults to `"src"`.
- **`files_to_ignore`**: List of files you don't want to copy over in the build.  
- **`import_aliases`**: Key-value pairs for rewriting import statements.

## Example Workflow

1. Put your "hacky" code in `_ghosti/Counter.sol`, referencing storage or test logic:
   ```solidity
   import "@/_ghosti/GhostiBase.sol";

   contract Counter is GhostiBase {
       // ...
       function add(uint256 x) public {
           // ...
           getGhostiStorage().ghosti_numbersSum += x;
       }
   }
   ```
2. Write your Foundry or other tests. Interact with the contract's storage directly.
3. When it's time for production:
   ```bash
   ghosti build
   ```
   This eradicates all Ghosti references and moves a clean `Counter.sol` to `src/Counter.sol`. 
4. Deploy from `src/`, free from debug code.

## Project Layout

A typical layout might look like:

```bash
repo/
├── ghosti/
│ ├── Counter.sol
│ ├── ICounter.sol
│ └── GhostiBase.sol
└── src/
└── test/
```

## Contributing

Ghosti is open source and welcomes contributions! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for details.

