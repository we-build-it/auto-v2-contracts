# Auto-Fee-Manager: Thornode Usage Examples

This document contains practical examples of how to use the auto-fee-manager contract on Thorchain Stagenet using the Thornode CLI.

## Environment Setup

### Test Addresses
- **Creator**: `sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh` (WBI-dev-1)
- **User**: `sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9` (multisig-ale)
- **Crank/WorkflowMgr/Destinations**: `sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3` (TCY testing)

### Network Configuration
- **Chain ID**: `thorchain-stagenet-2`
- **RPC Node**: `https://stagenet-rpc.ninerealms.com:443`

## Contract Deployment

### 1. Optimize the Contract

```bash
cargo run-script optimize
```

### 2. Upload the Contract (Store)

```bash
thornode tx wasm store /Users/araiczyk/workspace/auto-v2-contracts/artifacts/auto_fee_manager.wasm \
  --from multisig-ale \
  --chain-id thorchain-stagenet-2 \
  --node https://stagenet-rpc.ninerealms.com:443 \
  --gas auto --gas-adjustment 1.5
```

**Resulting Code ID**: `489`

### 3. Instantiate the Contract

```bash
thornode tx wasm instantiate 489 \
'{
  "max_debt": {
    "denom": "tcy",
    "amount": "100000000"
  },
  "min_balance_threshold": {
    "denom": "tcy", 
    "amount": "200000000"
  },
  "execution_fees_destination_address": "sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3",
  "distribution_fees_destination_address": "sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3",
  "accepted_denoms": ["tcy"],
  "crank_authorized_address": "sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3",
  "workflow_manager_address": "sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3",
  "creator_distribution_fee": "5"
}' \
--from multisig-ale \
--label "auto-fee-manager" \
--admin multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

**Contract address**: `sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w`

### 4. Migrate the Contract (Optional)

```bash
thornode tx wasm migrate sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
490 \
'{}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## User Operations

### 1. Deposit Funds

```bash
thornode tx wasm execute sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{"deposit":{}}' \
--amount 100000000tcy \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Query User Balances

```bash
thornode query wasm contract-state smart sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "get_user_balances": {
    "user": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9"
  }
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

### 3. Query Non-Creator Fees

```bash
thornode query wasm contract-state smart sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "get_non_creator_fees": {}
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

### 4. Query Creator Fees

```bash
thornode query wasm contract-state smart sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "get_creator_fees": {
    "creator": "sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh"
  }
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

## Crank/Workflow Manager Operations

### 1. Charge Fees from User Balance

```bash
thornode tx wasm execute sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "charge_fees_from_user_balance": {
    "batch": [
      {
        "user": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9",
        "fees": [
          {
            "workflow_instance_id": "1",
            "action_id": "check",
            "description": "Check if the user has something to claim",
            "timestamp": 1703123456,
            "amount": "100",
            "denom": "tcy",
            "fee_type": "execution"
          },
          {
            "workflow_instance_id": "1",
            "action_id": "test",
            "description": "Creator fee for very complicated logic",
            "timestamp": 1703123457,
            "amount": "50",
            "denom": "tcy",
            "fee_type": "creator",
            "creator_address": "sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh"
          }
        ]
      }
    ]
  }
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Charge Fees from Message Coins

```bash
thornode tx wasm execute \
sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "charge_fees_from_message_coins": {
    "fees": [
      {
        "workflow_instance_id": "1",
        "action_id": "claim",
        "description": "Creator fee for claiming tokens",
        "timestamp": 1703123457,
        "amount": "23",
        "denom": "tcy",
        "fee_type": "creator",
        "creator_address": "sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh"
      },
      {
        "workflow_instance_id": "1",
        "action_id": "stake",
        "description": "Creator fee for staking tokens",
        "timestamp": 1703123457,
        "amount": "66",
        "denom": "tcy",
        "fee_type": "creator",
        "creator_address": "sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh"
      }
    ]
  }
}' \
--from tcy-test \
--amount 89tcy \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto \
--gas-adjustment 1.5
```

## Distribution Operations

### 1. Distribute Creator Fees

```bash
thornode tx wasm execute \
sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "distribute_creator_fees": {}
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto \
--gas-adjustment 1.5
```

### 2. Distribute Non-Creator Fees

```bash
thornode tx wasm execute \
sthor16zs2laseyar5ffsk567gmqkrdynfv44u9jmn4a6gkln0vejfqvls5ktr5w \
'{
  "distribute_non_creator_fees": {}
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto \
--gas-adjustment 1.5
```

## Important Notes

1. **Gas Adjustment**: Use `--gas-adjustment 1.5` to avoid insufficient gas errors
2. **Denominations**: This example uses `tcy` as the main denom
3. **Authorization**: Only authorized addresses can execute crank and distribution operations
4. **Creator Distribution Fee**: A 5% (5 basis points) distribution fee is applied for creators
5. **Thresholds**: Minimum balance is 200,000,000 tcy and maximum debt is 100,000,000 tcy

## Typical Usage Flow

1. **Setup**: Deploy and instantiate the contract
2. **Deposit**: Users deposit funds
3. **Crank**: The workflow manager charges fees from balances or coins
4. **Distribution**: Fees are distributed to configured destinations
5. **Monitoring**: Query balances and accumulated fees 