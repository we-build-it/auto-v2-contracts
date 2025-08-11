import {
  WorkflowManagerClient,
  SDKConfig,
  createNewWorkflowMsg,
  createActionMsg,
  createTemplate,
  createNewInstanceMsg,
  createBigIntParam,
  createStringParam,
  WorkflowVisibility,
  ExecutionType,
  Coin
} from './index';

// Example configuration
const config: SDKConfig = {
  rpcUrl: 'https://rpc.cosmos.network:26657',
  contractAddress: 'cosmos1...', // Replace with actual contract address
  gasPrice: '0.025uatom',
  gasAdjustment: 1.1
};

async function example() {
  try {
    // Create a read-only client for queries
    console.log('Creating read-only client...');
    const readOnlyClient = await WorkflowManagerClient.createReadOnlyClient(config);
    
    // Create a signing client for transactions (replace with actual mnemonic)
    console.log('Creating signing client...');
    const signingClient = await WorkflowManagerClient.createSigningClient(
      config,
      'your twelve word mnemonic phrase here replace with actual mnemonic'
    );

    // Example 1: Create and publish a simple workflow
    console.log('\n=== Example 1: Publishing a Workflow ===');
    
    // Create a template for a simple action
    const template = createTemplate(
      'cosmos1...', // Replace with actual contract address
      JSON.stringify({
        transfer: {
          recipient: 'cosmos1...',
          amount: '1000000uatom'
        }
      }),
      [{ amount: '1000000', denom: 'uatom' }] as Coin[]
    );

    // Create an action
    const action = createActionMsg(
      {
        'recipient': { String: 'cosmos1...' },
        'amount': { BigInt: '1000000' }
      },
      [], // No next actions for this simple example
      { 'transfer_template': template },
      ['cosmos1...'] // Whitelisted contract
    );

    // Create the workflow
    const workflow = createNewWorkflowMsg(
      'simple-transfer-workflow',
      ['transfer_action'],
      ['transfer_action'],
      WorkflowVisibility.Public,
      {
        'transfer_action': action
      }
    );

    // Publish the workflow
    console.log('Publishing workflow...');
    const publishTxHash = await signingClient.publishWorkflow(workflow);
    console.log('Workflow published with tx hash:', publishTxHash);

    // Example 2: Execute a workflow instance
    console.log('\n=== Example 2: Executing a Workflow Instance ===');
    
    const instanceParams = {
      'user_recipient': { String: 'cosmos1...' },
      'user_amount': { BigInt: '500000' }
    };

    const instance = createNewInstanceMsg(
      'simple-transfer-workflow',
      instanceParams,
      ExecutionType.OneShot,
      new Date(Date.now() + 24 * 60 * 60 * 1000) // Expires in 24 hours
    );

    console.log('Executing workflow instance...');
    const executeTxHash = await signingClient.executeInstance(instance);
    console.log('Instance executed with tx hash:', executeTxHash);

    // Example 3: Query workflow data
    console.log('\n=== Example 3: Querying Workflow Data ===');
    
    // Get the workflow details
    const workflowData = await readOnlyClient.getWorkflowById('simple-transfer-workflow');
    console.log('Workflow details:', JSON.stringify(workflowData, null, 2));

    // Get instances for a user (replace with actual address)
    const userInstances = await readOnlyClient.getInstancesByRequester('cosmos1...');
    console.log('User instances:', JSON.stringify(userInstances, null, 2));

    // Example 4: Execute a specific action
    console.log('\n=== Example 4: Executing a Specific Action ===');
    
    const actionParams = {
      'custom_param': { String: 'custom_value' }
    };

    const actionTxHash = await signingClient.executeAction(
      'cosmos1...', // User address
      1, // Instance ID
      'transfer_action', // Action ID
      'transfer_template', // Template ID
      actionParams
    );
    console.log('Action executed with tx hash:', actionTxHash);

    // Example 5: Instance management
    console.log('\n=== Example 5: Instance Management ===');
    
    // Pause an instance
    console.log('Pausing instance...');
    const pauseTxHash = await signingClient.pauseInstance(1);
    console.log('Instance paused with tx hash:', pauseTxHash);

    // Resume an instance
    console.log('Resuming instance...');
    const resumeTxHash = await signingClient.resumeInstance(1);
    console.log('Instance resumed with tx hash:', resumeTxHash);

    // Cancel an instance
    console.log('Canceling instance...');
    const cancelTxHash = await signingClient.cancelInstance(1);
    console.log('Instance canceled with tx hash:', cancelTxHash);

    // Example 6: Complex workflow with multiple actions
    console.log('\n=== Example 6: Complex Workflow ===');
    
    // Create multiple templates
    const transferTemplate = createTemplate(
      'cosmos1...',
      JSON.stringify({ transfer: { recipient: 'cosmos1...', amount: '1000000uatom' } }),
      []
    );

    const stakeTemplate = createTemplate(
      'cosmos1...',
      JSON.stringify({ delegate: { validator: 'cosmosvaloper1...', amount: '500000uatom' } }),
      []
    );

    // Create actions with dependencies
    const transferAction = createActionMsg(
      { 'amount': { BigInt: '1000000' } },
      ['stake_action'], // Next action
      { 'transfer': transferTemplate },
      ['cosmos1...']
    );

    const stakeAction = createActionMsg(
      { 'validator': { String: 'cosmosvaloper1...' } },
      [], // No next actions (end of workflow)
      { 'stake': stakeTemplate },
      ['cosmos1...']
    );

    // Create complex workflow
    const complexWorkflow = createNewWorkflowMsg(
      'transfer-and-stake-workflow',
      ['transfer_action'],
      ['stake_action'],
      WorkflowVisibility.Private,
      {
        'transfer_action': transferAction,
        'stake_action': stakeAction
      }
    );

    console.log('Publishing complex workflow...');
    const complexTxHash = await signingClient.publishWorkflow(complexWorkflow);
    console.log('Complex workflow published with tx hash:', complexTxHash);

  } catch (error) {
    console.error('Error in example:', error);
  }
}

// Run the example
if (require.main === module) {
  example().catch(console.error);
}

export { example }; 