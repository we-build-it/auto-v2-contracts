use std::collections::{HashMap, HashSet};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp};

use crate::state::{Action, ActionParamValue, ExecutionType, Workflow, WorkflowInstance, WorkflowVisibility};

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
}

#[cw_serde]
pub struct NewWorkflowMsg {
    pub id: String,
    pub start_action: String,
    pub visibility: WorkflowVisibility,
    // action_name -> action
    pub actions: HashMap<String, Action>,
}

#[cw_serde]
pub struct NewInstanceMsg {
    pub workflow_id: String,
    pub onchain_parameters: HashMap<String, ActionParamValue>,
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
        instance_id: u64,
    },
    PauselInstance {
        instance_id: u64,
    },
    ResumeInstance {
        instance_id: u64,
    },
    ExecuteAction {
        user_address: String,
        instance_id: u64,
        action_id: String,
        params: Option<HashMap<String, String>>
    },
}

#[cw_serde]
pub struct GetInstancesResponse {
    pub flows: Vec<WorkflowInstance>,
}

#[cw_serde]
pub struct GetWorkflowResponse {
    pub template: Workflow,
}

#[cw_serde]
pub struct GetWorkflowInstanceResponse {
    pub instance: WorkflowInstance,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetInstancesResponse)]
    GetInstancesByRequester { requester_address: String },
    #[returns(GetWorkflowResponse)]
    GetWorkflowById { template_id: String },
    #[returns(GetWorkflowInstanceResponse)]
    GetWorkflowInstance { user_address: String, instance_id: u64 },
}

#[cw_serde]
pub struct MigrateMsg {
    // Add fields if you need to pass parameters during migration
}
