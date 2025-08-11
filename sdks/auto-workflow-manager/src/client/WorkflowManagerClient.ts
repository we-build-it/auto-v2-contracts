import {
  SigningCosmWasmClient,
  CosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { Coin, StdFee } from "@cosmjs/stargate";

import {
  SDKConfig,
  ExecuteOptions,
  ExecuteMsg,
  QueryMsg,
  SudoMsg,
  NewWorkflowMsg,
  NewInstanceMsg,
  GetWorkflowResponse,
  GetInstancesResponse,
  GetWorkflowInstanceResponse,
  WorkflowId,
  ActionId,
  InstanceId,
  TemplateId,
  WorkflowVisibility,
  ExecutionType,
} from "../types";
import {
  createBigIntParam,
  createStringParam,
  validateWorkflowId,
  validateActionId,
  validateInstanceId,
  validateAddress,
  getDefaultGas,
  formatErrorMessage,
} from "../utils/helpers";

export class WorkflowManagerClient {
  private client: SigningCosmWasmClient | CosmWasmClient;
  private contractAddress: string;
  private isReadOnly: boolean;
  private wallet?: DirectSecp256k1HdWallet;

  constructor(
    client: SigningCosmWasmClient | CosmWasmClient,
    contractAddress: string,
    wallet?: DirectSecp256k1HdWallet
  ) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.isReadOnly = !(client instanceof SigningCosmWasmClient);
    this.wallet = wallet;
  }

  /**
   * Create a read-only client for querying
   */
  static async createReadOnlyClient(config: SDKConfig): Promise<WorkflowManagerClient> {
    const client = await CosmWasmClient.connect(config.rpcUrl);
    return new WorkflowManagerClient(client, config.contractAddress);
  }

  /**
   * Create a signing client for executing transactions
   */
  static async createSigningClient(
    config: SDKConfig,
    mnemonic: string,
    options?: { prefix?: string }
  ): Promise<WorkflowManagerClient> {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      mnemonic,
      { prefix: options?.prefix || "cosmos" }
    );
    const client = await SigningCosmWasmClient.connectWithSigner(
      config.rpcUrl,
      wallet
    );
    return new WorkflowManagerClient(client, config.contractAddress, wallet);
  }

  /**
   * Publish a new workflow
   */
  async publishWorkflow(
    workflow: NewWorkflowMsg,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateWorkflowId(workflow.id)) {
      throw new Error("Invalid workflow ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      publish_workflow: { workflow },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to publish workflow: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Execute a workflow instance
   */
  async executeInstance(
    instance: NewInstanceMsg,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateWorkflowId(instance.workflow_id)) {
      throw new Error("Invalid workflow ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      execute_instance: { instance },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to execute instance: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Cancel a workflow instance
   */
  async cancelInstance(
    instanceId: InstanceId,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateInstanceId(instanceId)) {
      throw new Error("Invalid instance ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      cancel_instance: { instance_id: instanceId },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to cancel instance: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Pause a workflow instance
   */
  async pauseInstance(
    instanceId: InstanceId,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateInstanceId(instanceId)) {
      throw new Error("Invalid instance ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      pause_instance: { instance_id: instanceId },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to pause instance: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Resume a workflow instance
   */
  async resumeInstance(
    instanceId: InstanceId,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateInstanceId(instanceId)) {
      throw new Error("Invalid instance ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      resume_instance: { instance_id: instanceId },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to resume instance: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Execute a specific action in a workflow instance
   */
  async executeAction(
    userAddress: string,
    instanceId: InstanceId,
    actionId: ActionId,
    templateId: TemplateId,
    params?: Record<string, { String?: string; BigInt?: string }>,
    options?: ExecuteOptions
  ): Promise<string> {
    if (this.isReadOnly) {
      throw new Error("Cannot execute transactions with read-only client");
    }

    if (!validateAddress(userAddress)) {
      throw new Error("Invalid user address");
    }

    if (!validateInstanceId(instanceId)) {
      throw new Error("Invalid instance ID");
    }

    if (!validateActionId(actionId)) {
      throw new Error("Invalid action ID");
    }

    const signingClient = this.client as SigningCosmWasmClient;
    const accounts = await this.wallet!.getAccounts();
    const sender = accounts[0].address;

    const msg: ExecuteMsg = {
      execute_action: {
        user_address: userAddress,
        instance_id: instanceId,
        action_id: actionId,
        template_id: templateId,
        params,
      },
    };

    try {
      const result = await signingClient.execute(
        sender,
        this.contractAddress,
        msg,
        "auto"
      );
      return result.transactionHash;
    } catch (error) {
      throw new Error(`Failed to execute action: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Query instances by requester address
   */
  async getInstancesByRequester(requesterAddress: string): Promise<GetInstancesResponse> {
    if (!validateAddress(requesterAddress)) {
      throw new Error("Invalid requester address");
    }

    const msg: QueryMsg = {
      get_instances_by_requester: { requester_address: requesterAddress },
    };

    try {
      const result = await this.client.queryContractSmart(this.contractAddress, msg);
      return result as GetInstancesResponse;
    } catch (error) {
      throw new Error(`Failed to query instances: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Query workflow by ID
   */
  async getWorkflowById(workflowId: WorkflowId): Promise<GetWorkflowResponse> {
    if (!validateWorkflowId(workflowId)) {
      throw new Error("Invalid workflow ID");
    }

    const msg: QueryMsg = {
      get_workflow_by_id: { workflow_id: workflowId },
    };

    try {
      const result = await this.client.queryContractSmart(this.contractAddress, msg);
      return result as GetWorkflowResponse;
    } catch (error) {
      throw new Error(`Failed to query workflow: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Query workflow instance
   */
  async getWorkflowInstance(
    userAddress: string,
    instanceId: InstanceId
  ): Promise<GetWorkflowInstanceResponse> {
    if (!validateAddress(userAddress)) {
      throw new Error("Invalid user address");
    }

    if (!validateInstanceId(instanceId)) {
      throw new Error("Invalid instance ID");
    }

    const msg: QueryMsg = {
      get_workflow_instance: {
        user_address: userAddress,
        instance_id: instanceId,
      },
    };

    try {
      const result = await this.client.queryContractSmart(this.contractAddress, msg);
      return result as GetWorkflowInstanceResponse;
    } catch (error) {
      throw new Error(`Failed to query workflow instance: ${formatErrorMessage(error)}`);
    }
  }

  /**
   * Get contract address
   */
  getContractAddress(): string {
    return this.contractAddress;
  }

  /**
   * Check if client is read-only
   */
  isReadOnlyClient(): boolean {
    return this.isReadOnly;
  }

  /**
   * Get the underlying client
   */
  getClient(): SigningCosmWasmClient | CosmWasmClient {
    return this.client;
  }
} 