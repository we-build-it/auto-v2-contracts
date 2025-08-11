# Auto Workflow Manager SDK

TypeScript SDK for interacting with the Auto Workflow Manager smart contract on Cosmos/CosmWasm.

## Features

- 🔧 **Complete Contract Integration**: Full support for all contract functions
- 📝 **TypeScript Support**: Fully typed with comprehensive type definitions
- 🔐 **Secure**: Built on top of CosmJS for secure blockchain interactions
- 🚀 **Easy to Use**: Simple and intuitive API design
- 📚 **Well Documented**: Comprehensive documentation and examples

## Installation

```bash
npm install auto-workflow-manager-sdk
```

## Quick Start

### Basic Setup

```typescript
import { 
  WorkflowManagerClient, 
  SDKConfig,
  createNewWorkflowMsg,
  createActionMsg,
  createTemplate,
  WorkflowVisibility,
  ExecutionType
} from 'auto-workflow-manager-sdk';

// Configuration
const config: SDKConfig = {
  rpcUrl: 'https://rpc.cosmos.network:26657',
  contractAddress: 'cosmos1...', // Your contract address
  gasPrice: '0.025uatom',
  gasAdjustment: 1.1
};

// Create a read-only client for queries
const readOnlyClient = await WorkflowManagerClient.createReadOnlyClient(config);

// Create a signing client for transactions
const signingClient = await WorkflowManagerClient.createSigningClient(
  config,
  'your mnemonic phrase here'
);
```

### Publishing a Workflow

```typescript
// Create a template for an action
const template = createTemplate(
  'cosmos1...', // contract address
  JSON.stringify({ /* your contract message */ }),
  [] // optional funds
);

// Create an action
const action = createActionMsg(
  { // parameters
    'param1': { String: 'value1' },
    'param2': { BigInt: '1000000' }
  },
  ['next_action_id'], // next actions
  { 'template1': template }, // templates
  ['cosmos1...'] // whitelisted contracts
);

// Create the workflow
const workflow = createNewWorkflowMsg(
  'my-workflow-1',
  ['start_action'], // start actions
  ['end_action'], // end actions
  WorkflowVisibility.Public,
  {
    'action1': action
  }
);

// Publish the workflow
const txHash = await signingClient.publishWorkflow(workflow);
console.log('Workflow published:', txHash);
```

### Executing a Workflow Instance

```typescript
// Create instance parameters
const instanceParams = {
  'user_param': { String: 'user_value' },
  'amount': { BigInt: '500000' }
};

// Create instance message
const instance = createNewInstanceMsg(
  'my-workflow-1',
  instanceParams,
  ExecutionType.OneShot,
  new Date(Date.now() + 24 * 60 * 60 * 1000) // expires in 24 hours
);

// Execute the instance
const txHash = await signingClient.executeInstance(instance);
console.log('Instance executed:', txHash);
```

### Querying Data

```typescript
// Get all instances for a user
const instances = await readOnlyClient.getInstancesByRequester('cosmos1...');
console.log('User instances:', instances.instances);

// Get a specific workflow
const workflow = await readOnlyClient.getWorkflowById('my-workflow-1');
console.log('Workflow details:', workflow.workflow);

// Get a specific instance
const instance = await readOnlyClient.getWorkflowInstance('cosmos1...', 1);
console.log('Instance details:', instance.instance);
```

### Managing Instances

```typescript
// Pause an instance
await signingClient.pauseInstance(1);

// Resume an instance
await signingClient.resumeInstance(1);

// Cancel an instance
await signingClient.cancelInstance(1);
```

### Executing Actions

```typescript
// Execute a specific action
const actionParams = {
  'action_param': { String: 'action_value' }
};

const txHash = await signingClient.executeAction(
  'cosmos1...', // user address
  1, // instance ID
  'action1', // action ID
  'template1', // template ID
  actionParams // optional parameters
);
```

## API Reference

### WorkflowManagerClient

#### Static Methods

- `createReadOnlyClient(config: SDKConfig)`: Create a read-only client for queries
- `createSigningClient(config: SDKConfig, mnemonic: string, options?)`: Create a signing client for transactions

#### Instance Methods

##### Execute Methods
- `publishWorkflow(workflow: NewWorkflowMsg, options?)`: Publish a new workflow
- `executeInstance(instance: NewInstanceMsg, options?)`: Execute a workflow instance
- `cancelInstance(instanceId: number, options?)`: Cancel a workflow instance
- `pauseInstance(instanceId: number, options?)`: Pause a workflow instance
- `resumeInstance(instanceId: number, options?)`: Resume a workflow instance
- `executeAction(userAddress: string, instanceId: number, actionId: string, templateId: string, params?, options?)`: Execute a specific action

##### Query Methods
- `getInstancesByRequester(requesterAddress: string)`: Get all instances for a requester
- `getWorkflowById(workflowId: string)`: Get a specific workflow
- `getWorkflowInstance(userAddress: string, instanceId: number)`: Get a specific instance

### Utility Functions

- `createTemplate(contract: string, message: string, funds?)`: Create a template
- `createActionMsg(params, nextActions, templates, whitelistedContracts?)`: Create an action message
- `createNewWorkflowMsg(id, startActions, endActions, visibility, actions)`: Create a new workflow message
- `createNewInstanceMsg(workflowId, onchainParameters, executionType, expirationTime)`: Create a new instance message
- `createBigIntParam(value)`: Create a BigInt parameter
- `createStringParam(value)`: Create a String parameter
- `validateWorkflowId(id)`: Validate workflow ID format
- `validateActionId(id)`: Validate action ID format
- `validateInstanceId(id)`: Validate instance ID format
- `validateAddress(address)`: Validate address format

### Types

#### Enums
- `WorkflowVisibility`: Public | Private
- `ExecutionType`: OneShot | Recurrent
- `WorkflowState`: Approved | Pending
- `WorkflowInstanceState`: Running | Paused

#### Interfaces
- `SDKConfig`: SDK configuration
- `ExecuteOptions`: Transaction execution options
- `NewWorkflowMsg`: New workflow message
- `NewInstanceMsg`: New instance message
- `Template`: Action template
- `ActionMsg`: Action message

## Error Handling

The SDK provides comprehensive error handling with descriptive error messages:

```typescript
try {
  const txHash = await signingClient.publishWorkflow(workflow);
  console.log('Success:', txHash);
} catch (error) {
  console.error('Error:', error.message);
  // Error messages are automatically formatted for better readability
}
```

## Gas Estimation

The SDK provides default gas estimates for different operations:

- `publish_workflow`: 500,000 gas
- `execute_instance`: 300,000 gas
- `cancel_instance`: 100,000 gas
- `pause_instance`: 100,000 gas
- `resume_instance`: 100,000 gas
- `execute_action`: 200,000 gas

You can override these defaults by providing custom gas values in the options:

```typescript
await signingClient.publishWorkflow(workflow, {
  gas: '600000',
  gasAdjustment: 1.2
});
```

## Development

### Building

```bash
npm run build
```

### Development Mode

```bash
npm run dev
```

### Testing

```bash
npm test
```

## License

ISC

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Support

For support and questions, please open an issue on the GitHub repository. 