use std::collections::{HashMap, HashSet};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use cosmwasm_schema::cw_serde;

use crate::msg::{ActionId, ActionParamValue, ActionType, ExecutionType, InstanceId, ParamId, WorkflowId, WorkflowInstanceState, WorkflowState, WorkflowVisibility};

use crate::ContractError;

#[cw_serde]
pub struct Ownership {
    pub owner: Addr,
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
}

#[cw_serde]
pub struct Action {
    pub action_type: ActionType,
    pub next_actions: HashSet<String>,
    pub final_state: bool,
}

#[cw_serde]
pub struct Workflow {
    pub start_action: String,
    pub visibility: WorkflowVisibility,
    pub state: WorkflowState,
    pub publisher: Addr,
}


#[cw_serde]
pub struct WorkflowInstance {
    pub workflow_id: WorkflowId,
    pub state: WorkflowInstanceState,
    pub last_executed_action: Option<String>,
    pub execution_type: ExecutionType,
    pub expiration_time: Timestamp,
}


// =============================== 
// ========== WORKFLOWS ==========
// =============================== 

pub const WORKFLOWS: Map<WorkflowId, Workflow> = Map::new("w");
pub const WORKFLOW_ACTIONS: Map<(WorkflowId, ActionId), Action> = Map::new("wa");
pub const WORKFLOW_ACTION_PARAMS: Map<(WorkflowId, ActionId), HashMap<ParamId, ActionParamValue>> = Map::new("wap");

pub fn save_workflow(storage: &mut dyn Storage, id: &WorkflowId, workflow: &Workflow) -> StdResult<()> {
    WORKFLOWS.save(storage, id.clone(), workflow)
}

pub fn load_workflow(storage: &dyn Storage, workflow_id: &WorkflowId) -> StdResult<Workflow> {
    WORKFLOWS.load(storage, workflow_id.clone())
}

pub fn remove_workflow(storage: &mut dyn Storage, workflow_id: &WorkflowId) -> StdResult<()> {
    WORKFLOWS.remove(storage, workflow_id.clone());
    // Remove all actions for this workflow
    let actions = WORKFLOW_ACTIONS.prefix(workflow_id.clone()).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    for action_id in actions {
        remove_workflow_action(storage, workflow_id, &action_id)?;
    }
    Ok(())
}

pub fn save_workflow_action(storage: &mut dyn Storage, id: &WorkflowId, action_id: &ActionId, action: &Action) -> StdResult<()> {
    WORKFLOW_ACTIONS.save(storage, (id.clone(), action_id.clone()), action)
}

pub fn load_workflow_action(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<Action> {
    WORKFLOW_ACTIONS.load(storage, (workflow_id.clone(), action_id.clone()))
}

pub fn load_workflow_actions(storage: &dyn Storage, workflow_id: &WorkflowId) -> StdResult<HashMap<ActionId, Action>> {
    let actions = WORKFLOW_ACTIONS.prefix(workflow_id.clone()).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    let mut actions_map = HashMap::new();
    for action_id in actions {
        let action = load_workflow_action(storage, workflow_id, &action_id)?;
        actions_map.insert(action_id, action);
    }
    Ok(actions_map)
}

pub fn remove_workflow_action(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<()> {
    WORKFLOW_ACTIONS.remove(storage, (workflow_id.clone(), action_id.clone()));
    remove_workflow_action_params(storage, workflow_id, action_id)?;
    Ok(())
}

pub fn save_workflow_action_params(storage: &mut dyn Storage, id: &WorkflowId, action_id: &ActionId, params: &HashMap<ParamId, ActionParamValue>) -> StdResult<()> {
    WORKFLOW_ACTION_PARAMS.save(storage, (id.clone(), action_id.clone()), params)
}

pub fn load_workflow_action_params(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<HashMap<ParamId, ActionParamValue>> {
    WORKFLOW_ACTION_PARAMS.load(storage, (workflow_id.clone(), action_id.clone()))
}

pub fn remove_workflow_action_params(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<()> {
    WORKFLOW_ACTION_PARAMS.remove(storage, (workflow_id.clone(), action_id.clone()));
    Ok(())
}

// ========================================
// ========== WORKFLOW INSTANCES ==========
// ========================================

pub const WORKFLOW_INSTANCES: Map<(Addr, InstanceId), WorkflowInstance> = Map::new("wi");
pub const WORKFLOW_INSTANCE_PARAMS: Map<(Addr, InstanceId), HashMap<ParamId, ActionParamValue>>= Map::new("wip");

// requester_addr -> HashMap<instance_id, WorkflowInstance>

pub fn save_workflow_instance(storage: &mut dyn Storage, requester: &Addr, instance_id: &InstanceId, instance: &WorkflowInstance) -> StdResult<()> {
    WORKFLOW_INSTANCES.save(storage, (requester.clone(), instance_id.clone()), instance)
}

pub fn load_workflow_instance(storage: &dyn Storage, requester: &Addr, instance_id: &InstanceId) -> StdResult<WorkflowInstance> {
    WORKFLOW_INSTANCES.load(storage, (requester.clone(), instance_id.clone()))
}

pub fn remove_workflow_instance(storage: &mut dyn Storage, requester: &Addr, instance_id: &InstanceId) -> StdResult<()> {
    WORKFLOW_INSTANCES.remove(storage, (requester.clone(), instance_id.clone()));
    remove_workflow_instance_params(storage, requester, instance_id)?;
    Ok(())
}

pub fn load_workflow_instances_by_requester(storage: &dyn Storage, requester: &Addr) -> StdResult<HashMap<InstanceId, WorkflowInstance>> {
    let instances = WORKFLOW_INSTANCES.prefix(requester.clone()).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    let mut instances_map = HashMap::new();
    for instance_id in instances {
        let instance = load_workflow_instance(storage, requester, &instance_id)?;
        instances_map.insert(instance_id, instance);
    }
    Ok(instances_map)
}

pub fn save_workflow_instance_params(storage: &mut dyn Storage, requester: &Addr, instance_id: &InstanceId, params: &HashMap<ParamId, ActionParamValue>) -> StdResult<()> {
    WORKFLOW_INSTANCE_PARAMS.save(storage, (requester.clone(), instance_id.clone()), params)
}

pub fn load_workflow_instance_params(storage: &dyn Storage, requester: &Addr, instance_id: &InstanceId) -> StdResult<HashMap<ParamId, ActionParamValue>> {
    WORKFLOW_INSTANCE_PARAMS.load(storage, (requester.clone(), instance_id.clone()))
}

pub fn remove_workflow_instance_params(storage: &mut dyn Storage, requester: &Addr, instance_id: &InstanceId) -> StdResult<()> {
    WORKFLOW_INSTANCE_PARAMS.remove(storage, (requester.clone(), instance_id.clone()));
    Ok(())
}


// =============================== 
// ========== COUNTERS ==========
// =============================== 

pub const INSTANCE_COUNTER: Item<u64> = Item::new("instance_counter");

pub fn load_next_instance_id(storage: &mut dyn Storage) -> StdResult<u64> {
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
