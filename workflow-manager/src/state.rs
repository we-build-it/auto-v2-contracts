use std::collections::{HashMap, HashSet};

use cosmwasm_std::{Addr, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use cosmwasm_schema::cw_serde;

use crate::ContractError;

#[cw_serde]
pub struct Ownership {
    pub owner: Addr,
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
}

#[cw_serde]
pub enum ActionParamValue {
    String(String),
    BigInt(String), // Using String to represent BigInt for CosmWasm compatibility
}

#[cw_serde]
pub enum WorkflowVisibility {
    Public,
    Private,
}

#[cw_serde]
pub enum WorkflowState {
    Approved,
    Pending,
}

#[cw_serde]
pub enum ExecutionType {
    OneShot,
    Recurrent,
}

#[cw_serde]
pub enum WorkflowInstanceState {
    Running,
    Paused,
}

#[cw_serde]
pub enum ActionType {
    StakedTokenClaimer,
    TokenStaker,
}

#[cw_serde]
pub struct Action {
    pub action_type: ActionType,
    pub params: HashMap<String, ActionParamValue>,
    pub next_actions: HashSet<String>,
    pub final_state: bool,
}

#[cw_serde]
pub struct Workflow {
    pub id: String,
    pub start_action: String,
    pub visibility: WorkflowVisibility,
    pub state: WorkflowState,
    pub publisher: Addr,
    // action_name -> action
    pub actions: HashMap<String, Action>,
}

#[cw_serde]
pub struct WorkflowInstance {
    pub id: u64,
    pub state: WorkflowInstanceState,
    pub requester: Addr,
    pub last_executed_action: Option<String>,
    pub workflow_id: String,
    pub onchain_parameters: HashMap<String, ActionParamValue>,
    pub execution_type: ExecutionType,
    pub expiration_time: Timestamp,
}

// =============================== 
// ========== WORKFLOWS ==========
// =============================== 

pub const WORKFLOWS: Map<String, Workflow> = Map::new("workflows");

pub fn save_workflow(storage: &mut dyn Storage, workflow: &Workflow) -> StdResult<()> {
    WORKFLOWS.save(storage, workflow.id.clone(), workflow)
}

pub fn load_workflow(storage: &dyn Storage, workflow_id: &str) -> StdResult<Workflow> {
    WORKFLOWS.load(storage, workflow_id.to_string())
}

pub fn remove_workflow(storage: &mut dyn Storage, workflow_id: &str) -> StdResult<()> {
    WORKFLOWS.remove(storage, workflow_id.to_string());
    Ok(())
}

// ========================================
// ========== WORKFLOW INSTANCES ==========
// ========================================

// requester_addr -> HashMap<instance_id, WorkflowInstance>
pub const WORKFLOW_INSTANCES: Map<Addr, HashMap<u64, WorkflowInstance>> = Map::new("workflow_instances");

pub fn save_workflow_instance(storage: &mut dyn Storage, requester: Addr, instance: &WorkflowInstance) -> StdResult<()> {
    let mut instances = WORKFLOW_INSTANCES.load(storage, requester.clone()).unwrap_or_default();
    instances.insert(instance.id, instance.clone());
    WORKFLOW_INSTANCES.save(storage, requester, &instances)
}

pub fn load_workflow_instance(storage: &dyn Storage, requester: Addr, instance_id: u64) -> StdResult<WorkflowInstance> {
    let instances = WORKFLOW_INSTANCES.load(storage, requester)?;
    instances.get(&instance_id)
        .cloned()
        .ok_or_else(|| cosmwasm_std::StdError::not_found(format!("WorkflowInstance with id {}", instance_id)))
}

pub fn remove_workflow_instance(storage: &mut dyn Storage, requester: Addr, instance_id: u64) -> StdResult<()> {
    let mut instances = WORKFLOW_INSTANCES.load(storage, requester.clone())?;
    instances.remove(&instance_id);
    WORKFLOW_INSTANCES.save(storage, requester, &instances)
}

pub fn get_workflow_instances_by_requester(storage: &dyn Storage, requester: Addr) -> StdResult<HashMap<u64, WorkflowInstance>> {
    WORKFLOW_INSTANCES.load(storage, requester)
}

// =============================== 
// ========== COUNTERS ==========
// =============================== 

pub const INSTANCE_COUNTER: Item<u64> = Item::new("instance_counter");

pub fn get_next_instance_id(storage: &mut dyn Storage) -> StdResult<u64> {
    let current = INSTANCE_COUNTER.load(storage).unwrap_or(0);
    let next = current + 1;
    INSTANCE_COUNTER.save(storage, &next)?;
    Ok(next)
}

// =============================== 
// ========== OWNERSHIP ==========
// =============================== 

pub const OWNERSHIP: Item<Ownership> = Item::new("ownership");

pub fn save_ownership(storage: &mut dyn Storage, ownership: &Ownership) -> StdResult<()> {
    OWNERSHIP.save(storage, ownership)
}

pub fn load_ownership(storage: &dyn Storage) -> StdResult<Ownership> {
    OWNERSHIP.load(storage)
}

pub fn validate_sender_is_publisher(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), ContractError> {
    let state = load_ownership(storage)?;
    if !state.allowed_publishers.contains(&info.sender) {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_sender_is_action_executor(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), ContractError> {
    let state = load_ownership(storage)?;
    if !state.allowed_action_executors.contains(&info.sender) {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_sender_is_admin(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), ContractError> {
    let state = load_ownership(storage)?;
    if info.sender != state.owner {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}
