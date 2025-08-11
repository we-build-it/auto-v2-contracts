# Auto Workflow Manager - Architecture Brief

## Executive Summary

The Auto Workflow Manager is a CosmWasm smart contract that serves as the core orchestration engine for the AUTO automation system. It provides a secure, flexible, and scalable platform for managing automated workflows, executing workflow instances, and orchestrating complex multi-step actions across the blockchain ecosystem.

## Problem Statement

Traditional blockchain automation requires:
- Manual execution of each step
- Complex coordination between multiple contracts
- No standardized workflow management
- Limited reusability of automation patterns
- Security concerns with multi-step operations

## Solution Overview

The Auto Workflow Manager addresses these challenges by providing:

1. **Workflow Definition & Publishing**: Standardized workflow creation with reusable action templates
2. **Instance Management**: Lifecycle management for workflow executions
3. **Action Orchestration**: Secure execution of multi-step operations
4. **Parameter Resolution**: Dynamic parameter binding and template resolution
5. **Access Control**: Role-based security with whitelisting mechanisms

## Architecture Components

### 1. Message Data Structures

#### New Workflow Definition

Creators use this structure to define new workflows in the system.

```rust
pub struct NewWorkflowMsg {
    pub id: WorkflowId,                        // Hash of the entire workflow definition
    pub start_actions: HashSet<ActionId>,      // Actions where the workflow starts
    pub end_actions: HashSet<ActionId>,        // Action where the workflow ends
    pub visibility: WorkflowVisibility,        // Public or Private
    pub actions: HashMap<ActionId, ActionMsg>, // All the workflow's actions
}
```

#### Action definition
```rust
pub struct ActionMsg {
    pub params: HashMap<ParamId, ActionParamValue>, // Action parameters
    pub next_actions: HashSet<ActionId>,            // Allowed actions after this one
    pub templates: HashMap<TemplateId, Template>,   // Allowed templates for this action. The TemplateId is a hash over the template's fields.
    pub whitelisted_contracts: HashSet<String>,     // Allowed contracts to be called by this action
}
```

#### Template definition

This structure is used to define a CosmWasm contrall call. A `Template` is composed of three string templates that are used to define which contract will be called, using which message and with what funds. These string templates allow parameter replacement using the Action's paarameters.

```rust
pub struct Template {
    pub contract: String,             
    pub message: String,
    pub funds: Vec<(String, String)>, // (amount, denom)
}
```

#### Workflow Instantiation

Users use this structure to launch a new workflow instance to be executed by the system.

```rust
pub struct NewInstanceMsg {
    pub workflow_id: WorkflowId,                                // The workflow to launch
    pub onchain_parameters: HashMap<ParamId, ActionParamValue>, // Parameters for this instance
    pub execution_type: ExecutionType,                          // OneShot or Recurrent
    pub expiration_time: Timestamp,                             // When the instance must stop
}
```

#### Action Execution

The system uses this structure to execute the workflow instance actions

```rust
ExecuteAction {
    user_address: String,                              // User that launched the instance 
    instance_id: InstanceId,                           // Instance containing the action
    action_id: ActionId,                               // The action to execute
    template_id: TemplateId,                           // The template to use for the action
    params: Option<HashMap<ParamId, ActionParamValue>> // The remaining parameters to resolve the template
}
```

### 2. Storage Architecture

#### 2.1. Storage Data Structures

#### Workflows, Actions and Templates Definitions
```rust
pub struct Workflow {
    pub start_actions: HashSet<String>, // Entry point actions
    pub end_actions: HashSet<String>,   // Finish actions
    pub visibility: WorkflowVisibility, // Public/Private access control
    pub state: WorkflowState,           // Approval status
    pub publisher: Addr,                // Workflow owner
}
```
```rust
pub struct Action {
    pub next_actions: HashSet<String>, // Next allowed actions

}
```
```rust
pub struct Template {
    pub contract: String,             // To resolve the contract to call       
    pub message: String,              // To resolve the message to send
    pub funds: Vec<(String, String)>, // To resolve the funds to send (amount, denom)
}
```


#### Instance Management
```rust
pub struct WorkflowInstance {
    pub workflow_id: WorkflowId,              // Reference to workflow
    pub state: WorkflowInstanceState,         // Running/Paused
    pub last_executed_action: Option<String>, // Execution tracking
    pub execution_type: ExecutionType,        // OneShot/Recurrent
    pub expiration_time: Timestamp,           // TTL management
}
```

#### 2.2. Storage Data

The contract uses a hierarchical storage pattern with the following key mappings:

- **WORKFLOWS**: `Map<WorkflowId, Workflow>` - Core workflow definitions
- **WORKFLOW_ACTIONS**: `Map<(WorkflowId, ActionId), Action>` - Action definitions per workflow
- **WORKFLOW_ACTION_PARAMS**: `Map<(WorkflowId, ActionId), HashMap<ParamId, ActionParamValue>>` - Action parameters
- **WORKFLOW_ACTION_TEMPLATES**: `Map<(WorkflowId, ActionId, TemplateId), Template>` - Execution templates
- **WORKFLOW_INSTANCES**: `Map<(Addr, InstanceId), WorkflowInstance>` - User instance tracking
- **WORKFLOW_INSTANCE_PARAMS**: `Map<(Addr, InstanceId), HashMap<ParamId, ActionParamValue>>` - Instance-specific parameters

### 3. Security Model

#### Role-Based Access Control
- **Publishers**: Authorized to create and publish workflows
- **Action Executors**: Authorized to execute actions within workflows (initially only the Auto engine)
- **Admin**: Contract configuration management via sudo messages

#### Instance Isolation
- Users can only access their own workflow instances
- Private workflows restricted to publisher access
- Contract whitelisting for action execution (only allowed contracts can be executed)

#### Parameter Validation
- Template parameter resolution with security checks
- Dynamic parameter binding with validation
- Fund validation for action execution

## Key Features & Implementation

### 1. Workflow Publishing System

**Purpose**: Allow authorized publishers to create reusable workflow definitions

**Implementation**:
- Publisher authorization validation
- Workflow uniqueness enforcement
- Action template storage with parameter definitions
- Visibility control (Public/Private)

**Security**: Only authorized publishers can create workflows

### 2. Instance Execution Engine

**Purpose**: Create and manage workflow execution instances

**Implementation**:
- Auto-incremental instance ID generation
- Workflow approval validation
- Private workflow access control
- Instance parameter storage
- State management (Running/Paused)

**Security**: Instance ownership validation and workflow access control

### 3. Action Orchestration

**Purpose**: Execute individual actions within workflow instances

**Implementation**:
- Template-based execution with dynamic parameter resolution
- Contract whitelist validation
- Fund validation and forwarding
- State transition management
- Execution tracking

**Security**: Contract whitelisting and parameter validation

### 4. Parameter Resolution System

**Purpose**: Dynamic parameter binding and template resolution

**Resolution Patterns**:
- `#ip.requester` → User address executing the action
- `#ip.param_name` → Instance parameters
- `#cp.param_name` → Execution-time parameters
- Fixed values → Used as-is

**Implementation**:
- Recursive parameter resolution
- Template parameter substitution
- Fund resolution for action execution
- Error handling for invalid parameters

### 5. Lifecycle Management

**Purpose**: Complete workflow instance lifecycle control

**States**:
- **Running**: Active execution state
- **Paused**: Temporarily suspended
- **Cancelled**: Terminated execution

**Operations**:
- Pause/Resume instance execution
- Cancel instance with cleanup
- State validation for operations

## Integration Points

### 1. External Contract Integration
- Template-based contract calls
- Fund forwarding to external contracts
- Contract whitelist validation
- Error handling and rollback

### 2. Fee Management Integration
- Integration with Auto Fee Manager contract
- Fee collection and distribution
- Referral memo handling

### 3. AUTO Engine Integration
- Workflow orchestration coordination
- Instance state synchronization
- Action execution coordination

## Error Handling Strategy

### 1. Comprehensive Error Types
- Authorization errors (Unauthorized, InstanceAccessUnauthorized)
- Workflow errors (WorkflowNotFound, WorkflowNotApproved)
- Instance errors (InstanceNotFound, InstanceAlreadyExists)
- Action errors (ActionNotFound, TemplateNotFound)
- Validation errors (InvalidTemplate, InvalidDenom)

### 2. Error Propagation
- Structured error responses
- Detailed error messages for debugging
- Graceful failure handling
- State consistency maintenance

## Performance Considerations

### 1. Storage Optimization
- Efficient key-value storage patterns
- Minimal data duplication
- Cleanup procedures for terminated instances

### 2. Gas Optimization
- Batch operations where possible
- Efficient parameter resolution
- Minimal storage reads/writes

### 3. Scalability
- Auto-incremental IDs for instances
- Hierarchical storage structure
- Efficient query patterns

## Testing Strategy

### 1. Unit Tests
- Individual function testing
- Error condition validation
- State transition testing

### 2. Integration Tests
- End-to-end workflow execution
- Multi-step action testing
- Cross-contract integration

### 3. Security Tests
- Authorization validation
- Access control testing
- Contract whitelist validation

## Deployment & Configuration

### 1. Instantiation Parameters
```rust
pub struct InstantiateMsg {
    pub allowed_publishers: HashSet<Addr>,       // Initial publishers
    pub allowed_action_executors: HashSet<Addr>, // Initial executors
    pub referral_memo: String,                   // Fee referral
}
```

### 2. Admin Configuration
- Owner management via sudo messages
- Publisher/executor list updates
- Referral memo updates

## Future Enhancements

### 1. Planned Features
- Workflow versioning system
- Advanced parameter resolution patterns
- Cross-chain workflow support
- Enhanced monitoring and analytics

### 2. Scalability Improvements
- Batch action execution
- Parallel action processing
- Advanced caching mechanisms

## Conclusion

The Auto Workflow Manager provides a robust, secure, and scalable foundation for blockchain automation. Its modular architecture, comprehensive security model, and flexible parameter resolution system make it suitable for complex automation scenarios while maintaining simplicity for basic use cases.

The contract successfully addresses the core challenges of blockchain automation while providing the flexibility needed for future enhancements and integrations. 