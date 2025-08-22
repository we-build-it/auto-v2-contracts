// Types and interfaces
export type WorkflowId = string;
export type ActionId = string;
export type InstanceId = number;
export type ParamId = string;
export type TemplateId = string;

export enum WorkflowVisibility {
  Public = "Public",
  Private = "Private"
}

export enum ActionParamValue {
  String = "String",
  BigInt = "BigInt"
}

export interface ActionParamValueData {
  type: ActionParamValue;
  value: string;
}

export enum ExecutionType {
  OneShot = "OneShot",
  Recurrent = "Recurrent"
}

export enum WorkflowState {
  Approved = "Approved",
  Pending = "Pending"
}

export enum WorkflowInstanceState {
  Running = "Running",
  Paused = "Paused"
}

export interface Template {
  contract: string;
  message: string;
  funds: [string, string][]; // (amount, denom)
}

export interface Action {
  next_actions: Set<string>;
}

export interface Workflow {
  start_actions: Set<string>;
  end_actions: Set<string>;
  visibility: WorkflowVisibility;
  state: WorkflowState;
  publisher: string;
}

export interface WorkflowInstance {
  workflow_id: WorkflowId;
  state: WorkflowInstanceState;
  last_executed_action?: string;
  execution_type: ExecutionType;
  expiration_time: number; // Timestamp
}

export interface ExecuteActionParams {
  user_address: string;
  instance_id: InstanceId;
  action_id: ActionId;
  template_id: TemplateId;
  params?: Map<ParamId, ActionParamValueData>;
}

export interface ResolvedMessage {
  contract_addr: string;
  msg: string;
  funds: Coin[];
}

export interface Coin {
  amount: string;
  denom: string;
}

export interface ExecuteActionResult {
  messages: ResolvedMessage[];
  last_executed_action: string;
}

// Custom error class
export class ContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ContractError';
  }
}

// Main execute action function
export function executeAction(
  workflow: Workflow,
  userInstance: WorkflowInstance,
  action: Action,
  actionParams: Map<ParamId, ActionParamValueData>,
  instanceParams: Map<ParamId, ActionParamValueData>,
  template: Template,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined,
  currentTime: number,
  userAddress: string,
  currentActionId: string
): ExecuteActionResult {
  // Validate instance expiration time
  if (currentTime >= userInstance.expiration_time) {
    throw new ContractError("Instance has expired");
  }

  // Check if action can be executed
  const canExecute = checkActionExecution(
    workflow,
    userInstance,
    action,
    currentActionId
  );

  if (!canExecute) {
    throw new ContractError(
      "Action cannot be executed: not first execution, not valid next action, and not recurrent start action"
    );
  }

  // Resolve parameters
  const resolvedParams = resolveParameters(
    actionParams,
    userAddress,
    instanceParams,
    executeActionParams
  );

  // Execute template-based action
  const messages = executeDynamicTemplate(
    template,
    resolvedParams,
    executeActionParams
  );

  return {
    messages,
    last_executed_action: currentActionId
  };
}

// Check if action can be executed based on workflow state
function checkActionExecution(
  workflow: Workflow,
  userInstance: WorkflowInstance,
  action: Action,
  currentActionId: string
): boolean {
  if (!userInstance.last_executed_action) {
    // First execution - check if it's a start action
    return workflow.start_actions.has(currentActionId);
  }

  const lastExecutedAction = userInstance.last_executed_action;
  
  // Check if it's a valid next action
  if (action.next_actions.has(currentActionId)) {
    return true;
  }

  // Check if it's a recurrent start action (when last action was an end action)
  if (userInstance.execution_type === ExecutionType.Recurrent) {
    if (workflow.end_actions.has(lastExecutedAction) && 
        workflow.start_actions.has(currentActionId)) {
      return true;
    }
  }

  return false;
}

// Resolve parameter values based on different sources
function resolveParameters(
  actionParams: Map<ParamId, ActionParamValueData>,
  userAddress: string,
  instanceParams: Map<ParamId, ActionParamValueData>,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined
): Map<string, ActionParamValueData> {
  const resolvedParams = new Map<string, ActionParamValueData>();

  for (const [key, value] of actionParams) {
    const resolvedValue = resolveParamValue(
      value,
      userAddress,
      instanceParams,
      executeActionParams
    );
    resolvedParams.set(key, resolvedValue);
  }

  return resolvedParams;
}

// Resolve individual parameter value
function resolveParamValue(
  paramValue: ActionParamValueData,
  userAddress: string,
  instanceParams: Map<ParamId, ActionParamValueData>,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined
): ActionParamValueData {
  const valueStr = paramValue.value;

  if (valueStr === "#ip.requester") {
    return {
      type: ActionParamValue.String,
      value: userAddress
    };
  } else if (valueStr.startsWith("#ip.")) {
    // Extract the key after #ip.
    const key = valueStr.substring(4);
    const value = instanceParams.get(key);
    if (value) {
      return value;
    } else {
      throw new ContractError(`Parameter '${key}' not found in instance parameters`);
    }
  } else if (valueStr.startsWith("#cp.")) {
    // Extract the key after #cp.
    const key = valueStr.substring(4);
    if (executeActionParams) {
      const value = executeActionParams.get(key);
      if (value) {
        return value;
      } else {
        throw new ContractError(`Parameter '${key}' not found in execute action parameters`);
      }
    } else {
      throw new ContractError("Execute action parameters not provided");
    }
  } else {
    // Fixed value
    return paramValue;
  }
}

// Execute dynamic template and resolve placeholders
function executeDynamicTemplate(
  template: Template,
  resolvedParams: Map<string, ActionParamValueData>,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined
): ResolvedMessage[] {
  // Resolve template parameters
  const resolvedContract = resolveTemplateParameter(
    template.contract,
    resolvedParams,
    executeActionParams
  );
  
  const resolvedMessage = resolveTemplateParameter(
    template.message,
    resolvedParams,
    executeActionParams
  );
  
  const resolvedFunds = resolveTemplateFunds(
    template.funds,
    resolvedParams,
    executeActionParams
  );

  // Create the resolved message
  const resolvedMsg: ResolvedMessage = {
    contract_addr: resolvedContract,
    msg: resolvedMessage,
    funds: resolvedFunds
  };

  return [resolvedMsg];
}

// Resolve template parameter by replacing placeholders
function resolveTemplateParameter(
  templateParam: string,
  resolvedParams: Map<string, ActionParamValueData>,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined
): string {
  let result = templateParam;

  // Replace {{param}} placeholders with resolved values
  for (const [key, value] of resolvedParams) {
    const placeholder = `{{${key}}}`;
    result = result.replace(new RegExp(placeholder, 'g'), value.value);
  }

  // Replace #cp.param placeholders with execute action params
  if (executeActionParams) {
    for (const [key, value] of executeActionParams) {
      const placeholder = `#cp.${key}`;
      result = result.replace(new RegExp(placeholder, 'g'), value.value);
    }
  }

  return result;
}

// Resolve template funds
function resolveTemplateFunds(
  templateFunds: [string, string][],
  resolvedParams: Map<string, ActionParamValueData>,
  executeActionParams: Map<ParamId, ActionParamValueData> | undefined
): Coin[] {
  const resolvedFunds: Coin[] = [];

  for (const [amountTemplate, denomTemplate] of templateFunds) {
    const resolvedAmount = resolveTemplateParameter(
      amountTemplate,
      resolvedParams,
      executeActionParams
    );
    
    const resolvedDenom = resolveTemplateParameter(
      denomTemplate,
      resolvedParams,
      executeActionParams
    );

    resolvedFunds.push({
      amount: resolvedAmount,
      denom: resolvedDenom
    });
  }

  return resolvedFunds;
}

// Utility function to convert ActionParamValueData to string
export function actionParamValueToString(param: ActionParamValueData): string {
  return param.value;
}

// Utility function to create ActionParamValueData from string
export function createStringParam(value: string): ActionParamValueData {
  return {
    type: ActionParamValue.String,
    value
  };
}

// Utility function to create ActionParamValueData from BigInt
export function createBigIntParam(value: string): ActionParamValueData {
  return {
    type: ActionParamValue.BigInt,
    value
  };
}

// Example usage function
export function exampleExecuteAction(): void {
  // Example data
  const workflow: Workflow = {
    start_actions: new Set(["action1"]),
    end_actions: new Set(["action3"]),
    visibility: WorkflowVisibility.Public,
    state: WorkflowState.Approved,
    publisher: "publisher_address"
  };

  const userInstance: WorkflowInstance = {
    workflow_id: "workflow1",
    state: WorkflowInstanceState.Running,
    execution_type: ExecutionType.OneShot,
    expiration_time: Date.now() + 3600000 // 1 hour from now
  };

  const action: Action = {
    next_actions: new Set(["action2"])
  };

  const actionParams = new Map<ParamId, ActionParamValueData>([
    ["recipient", createStringParam("#ip.recipient")],
    ["amount", createBigIntParam("1000000")]
  ]);

  const instanceParams = new Map<ParamId, ActionParamValueData>([
    ["recipient", createStringParam("user123")]
  ]);

  const template: Template = {
    contract: "{{contract_addr}}",
    message: '{"transfer": {"recipient": "{{recipient}}", "amount": "{{amount}}"}}',
    funds: [["{{amount}}", "{{denom}}"]]
  };

  try {
    const result = executeAction(
      workflow,
      userInstance,
      action,
      actionParams,
      instanceParams,
      template,
      undefined,
      Date.now(),
      "user_address",
      "action1"
    );
    
    console.log("Execute action result:", result);
  } catch (error) {
    console.error("Execute action error:", error);
  }
} 