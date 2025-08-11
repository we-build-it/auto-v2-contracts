// Type definitions for Auto Workflow Manager Contract
// Based on the Rust contract message types

export type WorkflowId = string;
export type ActionId = string;
export type InstanceId = number;
export type ParamId = string;
export type TemplateId = string;

export enum WorkflowVisibility {
  Public = "Public",
  Private = "Private",
}

export enum ActionParamValue {
  String = "String",
  BigInt = "BigInt",
}

export interface ActionParamValueData {
  String?: string;
  BigInt?: string;
}

export enum ExecutionType {
  OneShot = "OneShot",
  Recurrent = "Recurrent",
}

export enum WorkflowState {
  Approved = "Approved",
  Pending = "Pending",
}

export enum WorkflowInstanceState {
  Running = "Running",
  Paused = "Paused",
}

export interface Template {
  contract: string;
  message: string;
  funds: [string, string][]; // [amount, denom]
}

export interface ActionMsg {
  params: Record<ParamId, ActionParamValueData>;
  next_actions: ActionId[];
  templates: Record<TemplateId, Template>;
  whitelisted_contracts: string[];
}

export interface NewWorkflowMsg {
  id: WorkflowId;
  start_actions: ActionId[];
  end_actions: ActionId[];
  visibility: WorkflowVisibility;
  actions: Record<ActionId, ActionMsg>;
}

export interface NewInstanceMsg {
  workflow_id: WorkflowId;
  onchain_parameters: Record<ParamId, ActionParamValueData>;
  execution_type: ExecutionType;
  expiration_time: string; // ISO timestamp string
}

// Execute Messages
export interface PublishWorkflowMsg {
  publish_workflow: {
    workflow: NewWorkflowMsg;
  };
}

export interface ExecuteInstanceMsg {
  execute_instance: {
    instance: NewInstanceMsg;
  };
}

export interface CancelInstanceMsg {
  cancel_instance: {
    instance_id: InstanceId;
  };
}

export interface PauseInstanceMsg {
  pause_instance: {
    instance_id: InstanceId;
  };
}

export interface ResumeInstanceMsg {
  resume_instance: {
    instance_id: InstanceId;
  };
}

export interface ExecuteActionMsg {
  execute_action: {
    user_address: string;
    instance_id: InstanceId;
    action_id: ActionId;
    template_id: TemplateId;
    params?: Record<ParamId, ActionParamValueData>;
  };
}

export type ExecuteMsg = 
  | PublishWorkflowMsg
  | ExecuteInstanceMsg
  | CancelInstanceMsg
  | PauseInstanceMsg
  | ResumeInstanceMsg
  | ExecuteActionMsg;

// Sudo Messages
export interface SetOwnerMsg {
  set_owner: string;
}

export interface SetAllowedPublishersMsg {
  set_allowed_publishers: string[];
}

export interface SetAllowedActionExecutorsMsg {
  set_allowed_action_executors: string[];
}

export interface SetReferralMemoMsg {
  set_referral_memo: string;
}

export type SudoMsg = 
  | SetOwnerMsg
  | SetAllowedPublishersMsg
  | SetAllowedActionExecutorsMsg
  | SetReferralMemoMsg;

// Query Messages
export interface GetInstancesByRequesterMsg {
  get_instances_by_requester: {
    requester_address: string;
  };
}

export interface GetWorkflowByIdMsg {
  get_workflow_by_id: {
    workflow_id: string;
  };
}

export interface GetWorkflowInstanceMsg {
  get_workflow_instance: {
    user_address: string;
    instance_id: number;
  };
}

export type QueryMsg = 
  | GetInstancesByRequesterMsg
  | GetWorkflowByIdMsg
  | GetWorkflowInstanceMsg;

// Response Types
export interface WorkflowResponse {
  id: WorkflowId;
  start_actions: ActionId[];
  end_actions: ActionId[];
  visibility: WorkflowVisibility;
  actions: Record<ActionId, ActionMsg>;
  publisher: string;
  state: WorkflowState;
}

export interface GetWorkflowResponse {
  workflow: WorkflowResponse;
}

export interface WorkflowInstanceResponse {
  workflow_id: WorkflowId;
  onchain_parameters: Record<ParamId, ActionParamValueData>;
  execution_type: ExecutionType;
  expiration_time: string;
  id: InstanceId;
  state: WorkflowInstanceState;
  requester: string;
  last_executed_action?: string;
}

export interface GetInstancesResponse {
  instances: WorkflowInstanceResponse[];
}

export interface GetWorkflowInstanceResponse {
  instance: WorkflowInstanceResponse;
}

// Configuration
export interface Config {
  owner: string;
  allowed_publishers: string[];
  allowed_action_executors: string[];
  referral_memo: string;
}

// SDK Configuration
export interface SDKConfig {
  rpcUrl: string;
  contractAddress: string;
  gasPrice?: string;
  gasAdjustment?: number;
}

export interface ExecuteOptions {
  gas?: string;
  gasAdjustment?: number;
  fee?: string;
  memo?: string;
} 