# Auto Workflow Manager

A CosmWasm smart contract for managing automated workflows, workflow execution, and action orchestration in the AUTO automation system.

## Overview

The Auto Workflow Manager is the core component of the AUTO automation system that handles workflow lifecycle management, instance execution, and action orchestration. It provides a secure and flexible platform for publishing, executing, and managing automated workflows with support for both public and private workflows.

## Features

### 📋 Workflow Management
- **Workflow Publishing**: Authorized publishers can create and publish workflows with defined actions and parameters.
- **Visibility Control**: Support for both public and private workflows with appropriate access controls.
- **Action Orchestration**: Define complex workflows with multiple actions, dependencies, and execution paths.
- **Parameter Resolution**: Dynamic parameter resolution with support for instance parameters, user context, and execution-time parameters.

### ⚡ Instance Execution
- **Instance Lifecycle**: Complete lifecycle management from creation to completion or cancellation.
- **Execution Types**: Support for both one-shot and recurrent workflow executions.
- **State Management**: Track instance states (Running, Paused) with appropriate state transitions.
- **Expiration Control**: Configurable expiration times for workflow instances.

### 🔧 Action Execution
- **Action Types**: Support for different action types including TokenStaker and StakedTokenClaimer.
- **Parameter Validation**: Comprehensive parameter validation and resolution.
- **Execution Control**: Enforce proper action sequencing and workflow state validation.
- **External Integration**: Secure integration with external contracts and services.

### 🔐 Authorization & Security
- **Role-based Access**: Separate roles for publishers and action executors.
- **Workflow Access Control**: Private workflows restricted to authorized users.
- **Instance Ownership**: Users can only manage their own workflow instances.
- **Sudo Administration**: Admin capabilities for contract configuration updates.

## Quick Start

1. **Instantiate** the contract with authorized publishers and executors
2. **Publish** workflows with defined actions and parameters
3. **Execute** workflow instances with appropriate parameters
4. **Manage** instance lifecycle (pause, resume, cancel)
5. **Execute** individual actions within workflow instances

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
    pub referral_memo: String,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    PublishWorkflow {
        workflow: NewWorkflowMsg,
    },
    ExecuteInstance {
        instance: NewInstanceMsg,
    },
    CancelInstance {
        instance_id: InstanceId,
    },
    PauseInstance {
        instance_id: InstanceId,
    },
    ResumeInstance {
        instance_id: InstanceId,
    },
    ExecuteAction {
        user_address: String,
        instance_id: InstanceId,
        action_id: ActionId,
        params: Option<HashMap<ParamId, ActionParamValue>>
    },
}
```

### Query

```rust
pub enum QueryMsg {
    GetInstancesByRequester { requester_address: String },
    GetWorkflowById { workflow_id: String },
    GetWorkflowInstance { user_address: String, instance_id: u64 },
}
```

### Sudo

```rust
pub enum SudoMsg {
    SetOwner(Addr),
    SetAllowedPublishers(HashSet<Addr>),
    SetAllowedActionExecutors(HashSet<Addr>),
    SetReferralMemo(String),
}
```

## Data Structures

```rust
pub struct NewWorkflowMsg {
    pub id: WorkflowId,
    pub start_actions: HashSet<ActionId>,
    pub end_actions: HashSet<ActionId>,
    pub visibility: WorkflowVisibility,
    pub actions: HashMap<ActionId, ActionMsg>,
}

pub struct ActionMsg {
    pub action_type: ActionType,
    pub params: HashMap<ParamId, ActionParamValue>,
    pub next_actions: HashSet<ActionId>,
}

pub struct NewInstanceMsg {
    pub workflow_id: WorkflowId,
    pub onchain_parameters: HashMap<ParamId, ActionParamValue>,
    pub execution_type: ExecutionType,
    pub expiration_time: Timestamp,
}

pub enum WorkflowVisibility {
    Public,
    Private,
}

pub enum ExecutionType {
    OneShot,
    Recurrent,
}

pub enum WorkflowState {
    Approved,
    Pending,
}

pub enum WorkflowInstanceState {
    Running,
    Paused,
}

pub enum ActionParamValue {
    String(String),
    BigInt(String),
}

pub enum ActionType {
    StakedTokenClaimer,
    TokenStaker,
}
```

## Events

- `publish_workflow` — When a workflow is successfully published.
- `execute_instance` — When a workflow instance is created and started.
- `cancel_instance` — When a workflow instance is cancelled.
- `pause_instance` — When a workflow instance is paused.
- `resume_instance` — When a workflow instance is resumed.
- `execute_action` — When an action within a workflow instance is executed.

## Usage Examples

> **📖 Thornode Usage Examples**: For complete usage examples with the Thornode CLI on Thorchain Stagenet, see [THORNODE_EXAMPLES.md](./THORNODE_EXAMPLES.md).

### 1. Publish a workflow

```rust
let workflow = NewWorkflowMsg {
    id: "staking_workflow".to_string(),
    start_actions: HashSet::from([
        "stake_tokens".to_string(),
    ]),
    end_actions: HashSet::from([
        "stake_tokens".to_string(),
    ]),
    visibility: WorkflowVisibility::Public,
    actions: HashMap::from([
        (
            "stake_tokens".to_string(),
            ActionMsg {
                action_type: ActionType::TokenStaker,
                params: HashMap::from([
                    ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
                    ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
                    ("userAddress".to_string(), ActionParamValue::String("#ip.requester".to_string())),
                    ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
                    ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
                ]),
                next_actions: HashSet::new()
            },
        ),
    ]),
};

let msg = ExecuteMsg::PublishWorkflow { workflow };
```

### 2. Execute a workflow instance

```rust
let instance = NewInstanceMsg {
    workflow_id: "staking_workflow".to_string(),
    onchain_parameters: HashMap::new(),
    execution_type: ExecutionType::OneShot,
    expiration_time: Timestamp::from_seconds(1000000000),
};

let msg = ExecuteMsg::ExecuteInstance { instance };
```

### 3. Execute an action within a workflow instance

```rust
let msg = ExecuteMsg::ExecuteAction {
    user_address: "user123".to_string(),
    instance_id: 1,
    action_id: "stake_tokens".to_string(),
    params: Some(HashMap::from([
        ("extra_param".to_string(), ActionParamValue::String("extra_value".to_string())),
    ])),
};
```

### 4. Pause a workflow instance

```rust
let msg = ExecuteMsg::PauseInstance { instance_id: 1 };
```

### 5. Resume a workflow instance

```rust
let msg = ExecuteMsg::ResumeInstance { instance_id: 1 };
```

### 6. Cancel a workflow instance

```rust
let msg = ExecuteMsg::CancelInstance { instance_id: 1 };
```

### 7. Query workflow instances by requester

```rust
let msg = QueryMsg::GetInstancesByRequester {
    requester_address: "user123".to_string(),
};
```

### 8. Query workflow by ID

```rust
let msg = QueryMsg::GetWorkflowById {
    workflow_id: "staking_workflow".to_string(),
};
```

### 9. Query specific workflow instance

```rust
let msg = QueryMsg::GetWorkflowInstance {
    user_address: "user123".to_string(),
    instance_id: 1,
};
```

## Parameter Resolution

The contract supports dynamic parameter resolution with the following patterns:

- `#ip.requester` - Resolves to the user address executing the action
- `#ip.param_name` - Resolves to instance parameters
- `#cp.param_name` - Resolves to execution-time parameters
- Fixed values - Used as-is without resolution

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

# Specific test suites
cargo test workflows
cargo test instances
cargo test actions
```

## Security

The contract implements comprehensive security measures:

- **Authorization Controls**: Strict role-based access for publishers and executors
- **Instance Isolation**: Users can only access their own workflow instances
- **Parameter Validation**: All parameters are validated before execution
- **State Validation**: Proper state transitions are enforced
- **Expiration Control**: Instances expire automatically to prevent resource exhaustion
- **Action Sequencing**: Proper action execution order is enforced

For security questions or vulnerability disclosures, contact the development team.

## Integration

The Auto Workflow Manager integrates with:

- **Auto Fee Manager**: For fee collection and distribution
- **External Contracts**: For executing specific actions (staking, claiming, etc.)
- **AUTO Engine**: For workflow orchestration and execution

## License

This project is licensed under the same terms as the AUTO automation system.
