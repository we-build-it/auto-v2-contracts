# JSON Examples for Auto V2 Contracts

This directory contains example JSON files for all possible executions and queries in the Auto V2 Contracts.

## Instantiate Messages

### `instantiate.json`
Example for contract initialization with allowed publishers and action executors.

## Execute Messages

### `execute-request-for-approval.json`
Example for requesting workflow approval. Contains a complete workflow template with:
- Workflow ID and start action
- Visibility setting (public/private)
- Actions with parameters, next actions, and final state flags

### `execute-instance.json`
Example for executing a workflow instance. Contains:
- Workflow ID reference
- On-chain parameters (user_wallet, stake_amount, reward_token)
- Execution type (oneshot/recurrent)
- Expiration timestamp

### `execute-cancel-instance.json`
Example for canceling a workflow instance by ID.

### `execute-pause-instance.json`
Example for pausing a running workflow instance.

### `execute-resume-instance.json`
Example for resuming a paused workflow instance.

### `execute-action.json`
Example for executing a specific action within a workflow instance. Contains:
- User address
- Instance ID
- Action ID to execute
- Optional custom parameters

## Query Messages

### `query-get-instances.json`
Example for querying all instances by requester address.

### `query-get-workflow.json`
Example for querying a specific workflow by template ID.

### `query-get-workflow-instance.json`
Example for querying a specific workflow instance by user address and instance ID.

## Usage

These JSON files can be used with:
- CosmWasm CLI tools
- Frontend applications
- Testing frameworks
- Documentation

## Parameter Types

- **String**: Simple string values
- **BigInt**: Large integer values stored as strings
- **Dynamic References**:
  - `#ip.requester`: Instance parameter for requester address
  - `#ip.key`: Instance parameter lookup
  - `#cp.key`: Custom parameter from execute_action

## Action Types

- **TokenStaker**: For staking tokens
- **StakedTokenClaimer**: For claiming staked token rewards 