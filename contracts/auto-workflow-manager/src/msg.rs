use std::collections::{HashMap, HashSet};
use std::fmt;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};

use crate::state::{PaymentConfig};

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

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowState::Approved => write!(f, "approved"),
            WorkflowState::Pending => write!(f, "pending"),
        }
    }
}

#[cw_serde]
pub enum WorkflowInstanceState {
    Running,
    Paused,
    Finished,
    Cancelled
}

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
    pub referral_memo: String,
    pub fee_manager_address: Addr,
    pub allowance_denom: String,
}

pub type WorkflowId = String;
pub type ActionId = String;
pub type InstanceId = u64;
pub type ParamId = String;
pub type TemplateId = String;

#[cw_serde]
pub struct Template {
    pub contract: String,
    pub message: String,
    pub funds: Vec<(String, String)>, // (amount, denom)
}

#[cw_serde]
pub struct ActionMsg {
    pub params: HashMap<ParamId, ActionParamValue>,
    pub next_actions: HashSet<ActionId>,
    pub templates: HashMap<TemplateId, Template>, // Now required, not optional
    pub whitelisted_contracts: HashSet<String>, // Lista de contratos whitelisted por acción
}
#[cw_serde]
pub struct NewWorkflowMsg {
    pub id: WorkflowId,
    pub start_actions: HashSet<ActionId>,
    pub end_actions: HashSet<ActionId>,
    pub visibility: WorkflowVisibility,
    // action_name -> action
    pub actions: HashMap<ActionId, ActionMsg>,
}
  
#[cw_serde]
pub struct NewInstanceMsg {
    pub workflow_id: WorkflowId,
    pub onchain_parameters: HashMap<ParamId, ActionParamValue>,
    pub offchain_parameters: HashMap<ParamId, ActionParamValue>,
    pub execution_type: ExecutionType,
    pub cron_expression: Option<String>,
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
    CancelRun {
        instance_id: InstanceId,
    },
    CancelInstance {
        instance_id: InstanceId,
    },
    // CancelSchedule {
    //     instance_id: InstanceId,
    // },
    PauseSchedule {
        instance_id: InstanceId,
    },
    ResumeSchedule {
        instance_id: InstanceId,
    },
    ExecuteAction {
        user_address: String,
        instance_id: InstanceId,
        action_id: ActionId,
        template_id: TemplateId, // Now required, not optional
        params: Option<HashMap<ParamId, ActionParamValue>>
    },
    PurgeInstances {
        instance_ids: Vec<InstanceId>,
    },
    SetUserPaymentConfig {
        payment_config: PaymentConfig,
    },
    RemoveUserPaymentConfig {
    },
    ChargeFees {
        batch_id: String,
        prices: HashMap<String, Decimal>,
        fees: Vec<UserFee>,
    },
    // TODO: temporal AuthZ test, remove this
    TestAuthz { },
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
pub struct GetUserPaymentConfigResponse {
    pub payment_config: Option<PaymentConfig>,
}

#[cw_serde]
pub enum FeeType {
    Execution,
    Creator { instance_id: InstanceId },
}

impl fmt::Display for FeeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeeType::Execution => write!(f, "execution"),
            FeeType::Creator { instance_id } => write!(f, "creator_{}", instance_id),
        }
    }
}


#[cw_serde]
pub struct FeeTotal {
    pub denom: String,
    pub amount: Uint128,
    pub fee_type: FeeType,
}

#[cw_serde]
pub struct UserFee {
    pub address: String,
    pub totals: Vec<FeeTotal>,
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
    #[returns(GetUserPaymentConfigResponse)]
    GetUserPaymentConfig { user_address: String },
    #[returns(InstantiateMsg)]
    GetConfig {},
}

#[cw_serde]
pub struct MigrateMsg {
    // Add fields if you need to pass parameters during migration
}
