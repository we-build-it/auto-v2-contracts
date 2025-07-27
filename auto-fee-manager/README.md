
# Auto Fee Manager

A CosmWasm smart contract for managing prepaid balances, fee collection, and revenue distribution in the AUTO automation system.

## Overview

The Auto Fee Manager handles all aspects of fee logic for workflows executed by the AUTO Engine. It manages user balances (including debt), enforces denom policies, collects and distributes fees, and integrates securely with the Workflow Manager.

## Features

### 🏦 Balance Management
- **Multi-denom Support**: Users can deposit and withdraw in any accepted token.
- **Real-time balance tracking**: Tracks user balances per token type with negative balance support.
- **Debt tracking**: Users can operate with negative balances, up to a configurable max debt.
- **Threshold alerts**: Emits events when balances drop below a configurable minimum.

### 💸 Fee Collection
- **Flexible fee sources**: Fees can be charged from user balance or from coins attached to the message.
- **Fee types**:
  - `Execution`: Used for infrastructure, gas, LLM calls, etc.
  - `Creator`: Fees paid to strategy (workflow) creators.
- **Unified representation**: All fees are modeled as a `Fee` with metadata like action, instance, and timestamp.
- **Batch processing**: Efficient batch fee collection from multiple users.
- **Validation**: When using message coins, coins must match expected fees exactly.

### 📤 Fee Distribution
- **Execution fees** are aggregated and sent to a predefined execution destination.
- **Creator fees** are tracked per creator and denom, and can be claimed or distributed.
- **Subscription-based distribution**: Only creators who have opted in to fee distribution will receive their fees when `DistributeCreatorFees` is called.
- **Distribution-ready**: The contract supports configurable fee splits.

### 🔐 Authorization & Integration
- **Crank-safe**: Specific address authorized for automated fee collection.
- **Workflow-aware**: Enforces access from the Workflow Manager where appropriate.
- **Sudo override**: Admin access to update critical settings.

## Quick Start

1. **Instantiate** the contract with configuration
2. **Deposit** funds to user accounts  
3. **Charge fees** from balances or message coins
4. **Distribute** collected fees to destinations

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub max_debt: Coin,
    pub min_balance_threshold: Coin,
    pub execution_fees_destination_address: Addr,
    pub distribution_fees_destination_address: Addr,
    pub accepted_denoms: Vec<String>,
    pub crank_authorized_address: Addr,
    pub workflow_manager_address: Addr,
    pub creator_distribution_fee: Uint128,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    Deposit {},
    Withdraw { denom: String, amount: Uint128 },
    ChargeFeesFromUserBalance { batch: Vec<UserFees> },
    ChargeFeesFromMessageCoins { fees: Vec<Fee> },
    ClaimCreatorFees {},
    DistributeNonCreatorFees {},
    DistributeCreatorFees {},
    EnableCreatorFeeDistribution {},
    DisableCreatorFeeDistribution {},
}
```

### Query

```rust
pub enum QueryMsg {
    HasExceededDebtLimit { user: Addr },
    GetUserBalances { user: Addr },
    GetCreatorFees { creator: Addr },
    GetNonCreatorFees {},
    IsCreatorSubscribed { creator: Addr },
    GetSubscribedCreators {},
}
```

### Sudo

```rust
pub enum SudoMsg {
    SetCrankAuthorizedAddress { address: Addr },
    SetWorkflowManagerAddress { address: Addr },
    SetExecutionFeesDestinationAddress { address: Addr },
    SetDistributionFeesDestinationAddress { address: Addr },
    SetCreatorDistributionFee { fee: Uint128 },
}
```

## Data Structures

```rust
pub struct Fee {
    pub workflow_instance_id: String,
    pub action_id: String,
    pub description: String,
    pub timestamp: u64,
    pub amount: Uint128,
    pub denom: String,
    pub fee_type: FeeType,
    pub creator_address: Option<Addr>, // Only populated when fee_type = Creator
}

pub enum FeeType {
    Execution,
    Creator,
}

pub struct UserFees {
    pub user: Addr,
    pub fees: Vec<Fee>,
}
```

## Events

- `deposit_completed` — When a user’s balance turns positive after deposit.
- `balance_below_threshold` — Emitted when balance drops below configured minimum.
- `fees_charged` — Emitted with per-type breakdown after fee deduction.
- `creator_fees_claimed` — When a creator withdraws their fees.
- `fees_distributed` — When non-creator fees are sent to destinations.
- `enable_creator_fee_distribution` — When a creator enables fee distribution.
- `disable_creator_fee_distribution` — When a creator disables fee distribution.

## Usage Examples

> **📖 Thornode Usage Examples**: For complete usage examples with the Thornode CLI on Thorchain Stagenet, see [THORNODE_EXAMPLES.md](./THORNODE_EXAMPLES.md).

### 1. User deposits funds

```rust
let msg = ExecuteMsg::Deposit {};
let funds = vec![Coin::new(100_000_000, "uusdc")]; // 100 USDC
```

### 2. Charge fees from user balance

```rust
let fees = vec![UserFees {
    user: Addr::unchecked("user123"),
    fees: vec![Fee {
        workflow_instance_id: "workflow_001".to_string(),
        action_id: "action_001".to_string(),
        description: "Execution fee".to_string(),
        timestamp: 1234567890,
        amount: Uint128::new(1000), // 0.001 USDC
        denom: "uusdc".to_string(),
        fee_type: FeeType::Execution,
        creator_address: None,
    }],
}];
let msg = ExecuteMsg::ChargeFeesFromUserBalance { batch: fees };
```

### 3. Charge fees from coins

```rust
let msg = ExecuteMsg::ChargeFeesFromMessageCoins {
    fees: vec![Fee {
        workflow_instance_id: "workflow_002".to_string(),
        action_id: "action_002".to_string(),
        description: "Creator fee".to_string(),
        timestamp: 1234567899,
        amount: Uint128::new(5000), // 0.005 USDC
        denom: "uusdc".to_string(),
        fee_type: FeeType::Creator,
        creator_address: Some(Addr::unchecked("creator123")),
    }],
};
let funds = vec![Coin::new(5000, "uusdc")];
```

### 4. Claim creator fees

```rust
let msg = ExecuteMsg::ClaimCreatorFees {};
```

### 5. Enable/Disable creator fee distribution

```rust
// Enable creator fee distribution (creator must call this)
let msg = ExecuteMsg::EnableCreatorFeeDistribution {};

// Disable creator fee distribution (creator must call this)
let msg = ExecuteMsg::DisableCreatorFeeDistribution {};
```

### 6. Distribute fees

```rust
// Distribute execution and distribution fees
let msg = ExecuteMsg::DistributeNonCreatorFees {};

// Distribute creator fees (only to subscribed creators)
let msg = ExecuteMsg::DistributeCreatorFees {};
```

## Building

```bash
# Build contract
cargo build --target wasm32-unknown-unknown --release

# Optimize for deployment
cargo run-script optimize
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests
```

## Security

The contract is designed with security best practices:
- Strict role-based control
- Proper validation on all inputs
- No reentrancy vulnerabilities
- Granular fee tracking & event logging

For vulnerability disclosures or questions, contact the development team.
