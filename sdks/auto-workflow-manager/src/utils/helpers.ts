import { Coin } from "@cosmjs/stargate";
import { WorkflowVisibility, ExecutionType } from "../types";

/**
 * Convert a timestamp to ISO string format
 */
export function toISOTimestamp(timestamp: Date | number | string): string {
  if (typeof timestamp === "string") {
    return timestamp;
  }
  if (typeof timestamp === "number") {
    return new Date(timestamp).toISOString();
  }
  return timestamp.toISOString();
}

/**
 * Convert ISO timestamp string to Date object
 */
export function fromISOTimestamp(timestamp: string): Date {
  return new Date(timestamp);
}

/**
 * Convert coins array to the format expected by the contract
 */
export function formatCoins(coins: Coin[]): [string, string][] {
  return coins.map(coin => [coin.amount, coin.denom]);
}

/**
 * Parse coins from contract format to Coin array
 */
export function parseCoins(coins: [string, string][]): Coin[] {
  return coins.map(([amount, denom]) => ({ amount, denom }));
}

/**
 * Create a BigInt parameter value
 */
export function createBigIntParam(value: string | number | bigint): { BigInt: string } {
  return { BigInt: value.toString() };
}

/**
 * Create a String parameter value
 */
export function createStringParam(value: string): { String: string } {
  return { String: value };
}

/**
 * Validate workflow ID format
 */
export function validateWorkflowId(workflowId: string): boolean {
  return typeof workflowId === "string" && workflowId.length > 0;
}

/**
 * Validate action ID format
 */
export function validateActionId(actionId: string): boolean {
  return typeof actionId === "string" && actionId.length > 0;
}

/**
 * Validate instance ID format
 */
export function validateInstanceId(instanceId: number): boolean {
  return typeof instanceId === "number" && instanceId > 0;
}

/**
 * Validate address format (basic validation)
 */
export function validateAddress(address: string): boolean {
  return typeof address === "string" && address.length > 0;
}

/**
 * Create a template object
 */
export function createTemplate(
  contract: string,
  message: string,
  funds: Coin[] = []
): { contract: string; message: string; funds: [string, string][] } {
  return {
    contract,
    message,
    funds: formatCoins(funds),
  };
}

/**
 * Create an action message object
 */
export function createActionMsg(
  params: Record<string, { String?: string; BigInt?: string }>,
  nextActions: string[],
  templates: Record<string, { contract: string; message: string; funds: [string, string][] }>,
  whitelistedContracts: string[] = []
): {
  params: Record<string, { String?: string; BigInt?: string }>;
  next_actions: string[];
  templates: Record<string, { contract: string; message: string; funds: [string, string][] }>;
  whitelisted_contracts: string[];
} {
  return {
    params,
    next_actions: nextActions,
    templates,
    whitelisted_contracts: whitelistedContracts,
  };
}

/**
 * Create a new workflow message
 */
export function createNewWorkflowMsg(
  id: string,
  startActions: string[],
  endActions: string[],
  visibility: WorkflowVisibility,
  actions: Record<string, {
    params: Record<string, { String?: string; BigInt?: string }>;
    next_actions: string[];
    templates: Record<string, { contract: string; message: string; funds: [string, string][] }>;
    whitelisted_contracts: string[];
  }>
): {
  id: string;
  start_actions: string[];
  end_actions: string[];
  visibility: WorkflowVisibility;
  actions: Record<string, {
    params: Record<string, { String?: string; BigInt?: string }>;
    next_actions: string[];
    templates: Record<string, { contract: string; message: string; funds: [string, string][] }>;
    whitelisted_contracts: string[];
  }>;
} {
  return {
    id,
    start_actions: startActions,
    end_actions: endActions,
    visibility,
    actions,
  };
}

/**
 * Create a new instance message
 */
export function createNewInstanceMsg(
  workflowId: string,
  onchainParameters: Record<string, { String?: string; BigInt?: string }>,
  executionType: ExecutionType,
  expirationTime: Date | number | string
): {
  workflow_id: string;
  onchain_parameters: Record<string, { String?: string; BigInt?: string }>;
  execution_type: ExecutionType;
  expiration_time: string;
} {
  return {
    workflow_id: workflowId,
    onchain_parameters: onchainParameters,
    execution_type: executionType,
    expiration_time: toISOTimestamp(expirationTime),
  };
}

/**
 * Calculate default gas for different operations
 */
export function getDefaultGas(operation: string): string {
  const gasMap: Record<string, string> = {
    publish_workflow: "500000",
    execute_instance: "300000",
    cancel_instance: "100000",
    pause_instance: "100000",
    resume_instance: "100000",
    execute_action: "200000",
  };
  
  return gasMap[operation] || "200000";
}

/**
 * Format error message for better readability
 */
export function formatErrorMessage(error: any): string {
  if (typeof error === "string") {
    return error;
  }
  
  if (error?.message) {
    return error.message;
  }
  
  if (error?.error) {
    return typeof error.error === "string" ? error.error : JSON.stringify(error.error);
  }
  
  return JSON.stringify(error);
} 