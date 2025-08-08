# Auto-Workflow-Manager: Thornode Usage Examples

This document contains practical examples of how to use the auto-workflow-manager contract on Thorchain Stagenet using the Thornode CLI.

## Environment Setup

### Test Addresses
- **Publisher**: `sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh` (WBI-dev-1)
- **User**: `sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9` (multisig-ale)
- **Action Executor**: `sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3` (TCY testing)
- **Admin**: `sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9` (multisig-ale)

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
thornode tx wasm store /Users/araiczyk/workspace/auto-v2-contracts/artifacts/auto_workflow_manager.wasm \
  --from multisig-ale \
  --chain-id thorchain-stagenet-2 \
  --node https://stagenet-rpc.ninerealms.com:443 \
  --gas auto --gas-adjustment 1.5
```

**Resulting Code ID**: `491`

### 3. Instantiate the Contract

```bash
thornode tx wasm instantiate 491 \
'{
  "allowed_publishers": ["sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh"],
  "allowed_action_executors": ["sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3"],
  "referral_memo": "auto-workflow-manager-stagenet"
}' \
--from multisig-ale \
--label "auto-workflow-manager" \
--admin multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

**Contract address**: `sthor1workflowmanagercontractaddress123456789`

### 4. Migrate the Contract (Optional)

```bash
thornode tx wasm migrate sthor1workflowmanagercontractaddress123456789 \
492 \
'{}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## Publisher Operations

### 1. Publish a Public Workflow

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "publish_workflow": {
    "workflow": {
      "id": "staking_workflow",
      "start_actions": ["stake_tokens"],
      "end_actions": ["stake_tokens"],
      "visibility": "public",
      "actions": {
        "stake_tokens": {
          "action_type": "token_staker",
          "params": {
            "provider": "daodao",
            "contractAddress": "osmo1stakingcontract123456789",
            "userAddress": "#ip.requester",
            "amount": "1000000",
            "denom": "uosmo"
          },
          "next_actions": []
        }
      }
    }
  }
}' \
--from wbi-dev-1 \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Publish a Private Workflow

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "publish_workflow": {
    "workflow": {
      "id": "private_claiming_workflow",
      "start_actions": "claim_rewards",
      "end_actions": "claim_rewards",
      "visibility": "private",
      "actions": {
        "claim_rewards": {
          "action_type": "staked_token_claimer",
          "params": {
            "provider": "daodao",
            "contractAddress": "osmo1claimingcontract123456789",
            "userAddress": "#ip.requester",
            "amount": "500000"
          },
          "next_actions": []
        }
      }
    }
  }
}' \
--from wbi-dev-1 \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 3. Publish a Multi-Action Workflow

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "publish_workflow": {
    "workflow": {
      "id": "stake_and_claim_workflow",
      "start_actions": ["stake_tokens"],
      "end_actions": ["claim_rewards"],
      "visibility": "public",
      "actions": {
        "stake_tokens": {
          "action_type": "token_staker",
          "params": {
            "provider": "daodao",
            "contractAddress": "osmo1stakingcontract123456789",
            "userAddress": "#ip.requester",
            "amount": "1000000",
            "denom": "uosmo"
          },
          "next_actions": ["claim_rewards"]
        },
        "claim_rewards": {
          "action_type": "staked_token_claimer",
          "params": {
            "provider": "daodao",
            "contractAddress": "osmo1claimingcontract123456789",
            "userAddress": "#ip.requester",
            "amount": "500000"
          },
          "next_actions": []
        }
      }
    }
  }
}' \
--from wbi-dev-1 \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## User Operations

### 1. Execute a Workflow Instance

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_instance": {
    "instance": {
      "workflow_id": "staking_workflow",
      "onchain_parameters": {
        "custom_param": "custom_value"
      },
      "execution_type": "one_shot",
      "expiration_time": "1703123456"
    }
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Execute a Recurrent Workflow Instance

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_instance": {
    "instance": {
      "workflow_id": "stake_and_claim_workflow",
      "onchain_parameters": {
        "recurring_param": "recurring_value"
      },
      "execution_type": "recurrent",
      "expiration_time": "1703123456"
    }
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 3. Pause a Workflow Instance

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "pause_instance": {
    "instance_id": 1
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 4. Resume a Workflow Instance

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "resume_instance": {
    "instance_id": 1
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 5. Cancel a Workflow Instance

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "cancel_instance": {
    "instance_id": 1
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## Action Executor Operations

### 1. Execute an Action

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_action": {
    "user_address": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9",
    "instance_id": 1,
    "action_id": "stake_tokens",
    "params": {
      "extra_param": "extra_value"
    }
  }
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Execute an Action without Parameters

```bash
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_action": {
    "user_address": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9",
    "instance_id": 1,
    "action_id": "claim_rewards",
    "params": null
  }
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## Query Operations

### 1. Query Workflow by ID

```bash
thornode query wasm contract-state smart sthor1workflowmanagercontractaddress123456789 \
'{
  "get_workflow_by_id": {
    "workflow_id": "staking_workflow"
  }
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

### 2. Query Instances by Requester

```bash
thornode query wasm contract-state smart sthor1workflowmanagercontractaddress123456789 \
'{
  "get_instances_by_requester": {
    "requester_address": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9"
  }
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

### 3. Query Specific Workflow Instance

```bash
thornode query wasm contract-state smart sthor1workflowmanagercontractaddress123456789 \
'{
  "get_workflow_instance": {
    "user_address": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9",
    "instance_id": 1
  }
}' \
--node https://stagenet-rpc.ninerealms.com:443
```

## Admin Operations (Sudo)

### 1. Set Allowed Publishers

```bash
thornode tx wasm sudo sthor1workflowmanagercontractaddress123456789 \
'{
  "set_allowed_publishers": {
    "allowed_publishers": ["sthor1et07cf27p29fu2j4e52pajlkndzz6jt7h0amkh", "sthor1newpublisher123456789"]
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Set Allowed Action Executors

```bash
thornode tx wasm sudo sthor1workflowmanagercontractaddress123456789 \
'{
  "set_allowed_action_executors": {
    "allowed_action_executors": ["sthor1p9j4ae2pquuzey4c7kweqlg7ymxz2raxka4sn3", "sthor1newexecutor123456789"]
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 3. Set Referral Memo

```bash
thornode tx wasm sudo sthor1workflowmanagercontractaddress123456789 \
'{
  "set_referral_memo": {
    "referral_memo": "updated-referral-memo"
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## Parameter Resolution Examples

### 1. Instance Parameters (#ip)

```bash
# Publish workflow with instance parameter resolution
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "publish_workflow": {
    "workflow": {
      "id": "parameterized_workflow",
      "start_actions": ["custom_action"],
      "end_actions": ["custom_action"],
      "visibility": "public",
      "actions": {
        "custom_action": {
          "action_type": "token_staker",
          "params": {
            "provider": "daodao",
            "contractAddress": "#ip.contract_address",
            "userAddress": "#ip.requester",
            "amount": "#ip.stake_amount",
            "denom": "uosmo"
          },
          "next_actions": []
        }
      }
    }
  }
}' \
--from wbi-dev-1 \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5

# Execute instance with parameters
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_instance": {
    "instance": {
      "workflow_id": "parameterized_workflow",
      "onchain_parameters": {
        "contract_address": "osmo1customcontract123456789",
        "stake_amount": "2000000"
      },
      "execution_type": "one_shot",
      "expiration_time": "1703123456"
    }
  }
}' \
--from multisig-ale \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

### 2. Execution Parameters (#cp)

```bash
# Execute action with execution-time parameters
thornode tx wasm execute sthor1workflowmanagercontractaddress123456789 \
'{
  "execute_action": {
    "user_address": "sthor1an07qawyk49eg9a4hdvde509wprjtfl7qdxuv9",
    "instance_id": 1,
    "action_id": "custom_action",
    "params": {
      "execution_param": "execution_value",
      "dynamic_amount": "3000000"
    }
  }
}' \
--from tcy-test \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

## Important Notes

1. **Gas Adjustment**: Use `--gas-adjustment 1.5` to avoid insufficient gas errors
2. **Authorization**: Only authorized publishers can publish workflows and only authorized executors can execute actions
3. **Instance Ownership**: Users can only manage their own workflow instances
4. **Parameter Resolution**: 
   - `#ip.requester` resolves to the user address
   - `#ip.param_name` resolves to instance parameters
   - `#cp.param_name` resolves to execution-time parameters
5. **Workflow Visibility**: Private workflows can only be executed by the publisher
6. **Instance Expiration**: Instances automatically expire based on the configured expiration time

## Typical Usage Flow

1. **Setup**: Deploy and instantiate the contract with authorized publishers and executors
2. **Publish**: Publishers create and publish workflows with defined actions and parameters
3. **Execute**: Users execute workflow instances with appropriate parameters
4. **Manage**: Users can pause, resume, or cancel their workflow instances
5. **Execute Actions**: Authorized executors execute individual actions within workflow instances
6. **Monitor**: Query workflows, instances, and execution status

## Error Handling

Common error scenarios and their solutions:

- **Unauthorized Publisher**: Ensure the sender is in the allowed_publishers list
- **Unauthorized Executor**: Ensure the sender is in the allowed_action_executors list
- **Workflow Not Found**: Verify the workflow_id exists and is published
- **Instance Not Found**: Verify the instance_id exists and belongs to the user
- **Action Not Found**: Verify the action_id exists in the workflow
- **Instance Expired**: Check the expiration_time and create a new instance if needed
- **Invalid Action Sequence**: Ensure actions are executed in the correct order 