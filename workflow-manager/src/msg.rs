use std::collections::{HashMap, HashSet};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp};

#[cw_serde]
pub enum WorkflowVisibility {
    Public,
    Private,
}

#[cw_serde]
pub enum ActionParamValue {
    String(String),
    BigInt(String), // Using String to represent BigInt for CosmWasm compatibility
}

#[cw_serde]
pub enum ExecutionType {
    OneShot,
    Recurrent,
}

#[cw_serde]
pub enum WorkflowState {
    Approved,
    Pending,
}

#[cw_serde]
pub enum WorkflowInstanceState {
    Running,
    Paused,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
    pub referral_memo: String,
}

#[cw_serde]
pub enum ActionType {
    StakedTokenClaimer,
    TokenStaker,
}

pub type WorkflowId = String;
pub type ActionId = String;
pub type InstanceId = u64;
pub type ParamId = String;

#[cw_serde]
pub struct ActionMsg {
    pub action_type: ActionType,
    pub params: HashMap<ParamId, ActionParamValue>,
    pub next_actions: HashSet<ActionId>,
    pub final_state: bool,
}
#[cw_serde]
pub struct NewWorkflowMsg {
    pub id: WorkflowId,
    pub start_action: ActionId,
    pub visibility: WorkflowVisibility,
    // action_name -> action
    pub actions: HashMap<ActionId, ActionMsg>,
}
  
#[cw_serde]
pub struct NewInstanceMsg {
    pub workflow_id: WorkflowId,
    pub onchain_parameters: HashMap<ParamId, ActionParamValue>,
    pub execution_type: ExecutionType,
    pub expiration_time: Timestamp,
}

#[cw_serde]
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
    PauselInstance {
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

#[cw_serde]
pub enum SudoMsg {
    SetOwner(Addr),
    SetAllowedPublishers(HashSet<Addr>),
    SetAllowedActionExecutors(HashSet<Addr>),
    SetReferralMemo(String),
}

#[cw_serde]
pub struct WorkflowResponse {
    #[serde(flatten)]
    pub base: NewWorkflowMsg,
    pub publisher: Addr,
    pub state: WorkflowState,
}

#[cw_serde]
pub struct GetWorkflowResponse {
    pub workflow: WorkflowResponse,
}

#[cw_serde]
pub struct WorkflowInstanceResponse {
    #[serde(flatten)]
    pub base: NewInstanceMsg,
    pub id: InstanceId,
    pub state: WorkflowInstanceState,
    pub requester: Addr,
    pub last_executed_action: Option<String>,
}

#[cw_serde]
pub struct GetInstancesResponse {
    pub instances: Vec<WorkflowInstanceResponse>,
}

#[cw_serde]
pub struct GetWorkflowInstanceResponse {
    pub instance: WorkflowInstanceResponse,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetInstancesResponse)]
    GetInstancesByRequester { requester_address: String },
    #[returns(GetWorkflowResponse)]
    GetWorkflowById { workflow_id: String },
    #[returns(GetWorkflowInstanceResponse)]
    GetWorkflowInstance { user_address: String, instance_id: u64 },
}

#[cw_serde]
pub struct MigrateMsg {
    // Add fields if you need to pass parameters during migration
}
