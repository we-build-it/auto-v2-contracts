use std::collections::{HashMap, HashSet};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use cosmwasm_schema::cw_serde;

use crate::msg::{ActionId, ActionParamValue, ExecutionType, InstanceId, ParamId, WorkflowId, WorkflowInstanceState, WorkflowState, WorkflowVisibility, TemplateId, Template};

use crate::ContractError;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub allowed_publishers: HashSet<Addr>,
    pub allowed_action_executors: HashSet<Addr>,
    pub referral_memo: String,
    pub fee_manager_address: Addr,
}

#[cw_serde]
pub struct Action {
    pub next_actions: HashSet<String>,
}

#[cw_serde]
pub struct Workflow {
    pub start_actions: HashSet<String>,
    pub end_actions: HashSet<String>,
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
    // pub requester: Addr,
}

#[cw_serde]
pub struct PaymentConfig {
    pub allowance_usd: Uint128,
    pub source: PaymentSource,
}

#[cw_serde]
pub enum PaymentSource {
    Wallet,
    Prepaid,
}

// ==================================== 
// ========== PAYMENT CONFIG ==========
// ==================================== 

pub const USER_PAYMENT_CONFIG: Map<Addr, PaymentConfig> = Map::new("upc");

pub fn save_user_payment_config(storage: &mut dyn Storage, user: &Addr, config: &PaymentConfig) -> StdResult<()> {
    USER_PAYMENT_CONFIG.save(storage, user.clone(), config)
}

pub fn load_user_payment_config(storage: &dyn Storage, user: &Addr) -> StdResult<PaymentConfig> {
    USER_PAYMENT_CONFIG.load(storage, user.clone())
}

pub fn remove_user_payment_config(storage: &mut dyn Storage, user: &Addr) -> StdResult<()> {
    USER_PAYMENT_CONFIG.remove(storage, user.clone());
    Ok(())
}

// =============================== 
// ========== WORKFLOWS ==========
// =============================== 

pub const WORKFLOWS: Map<WorkflowId, Workflow> = Map::new("w");
pub const WORKFLOW_ACTIONS: Map<(WorkflowId, ActionId), Action> = Map::new("wa");
pub const WORKFLOW_ACTION_PARAMS: Map<(WorkflowId, ActionId), HashMap<ParamId, ActionParamValue>> = Map::new("wap");
pub const WORKFLOW_ACTION_TEMPLATES: Map<(WorkflowId, ActionId, TemplateId), Template> = Map::new("wat");
pub const WORKFLOW_ACTION_CONTRACTS: Map<(WorkflowId, ActionId, String), ()> = Map::new("wac");

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
    remove_workflow_action_templates(storage, workflow_id, action_id)?;
    remove_workflow_action_contracts(storage, workflow_id, action_id)?;
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

pub fn save_workflow_action_templates(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId, templates: &HashMap<   TemplateId, Template>) -> StdResult<()> {
    for (template_id, template) in templates {
        WORKFLOW_ACTION_TEMPLATES.save(storage, (workflow_id.clone(), action_id.clone(), template_id.clone()), template)?;
    }
    Ok(())
}

pub fn load_workflow_action_template(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId, template_id: &TemplateId) -> StdResult<Template> {
    WORKFLOW_ACTION_TEMPLATES.load(storage, (workflow_id.clone(), action_id.clone(), template_id.clone()))
}

pub fn load_workflow_action_templates(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<HashMap<TemplateId, Template>> {
    let templates = WORKFLOW_ACTION_TEMPLATES.prefix((workflow_id.clone(), action_id.clone())).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    let mut templates_map = HashMap::new();
    for template_id in templates {
        let template = load_workflow_action_template(storage, workflow_id, action_id, &template_id)?;
        templates_map.insert(template_id, template);
    }
    Ok(templates_map)
}

pub fn remove_workflow_action_templates(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<()> {
    let templates = WORKFLOW_ACTION_TEMPLATES.prefix((workflow_id.clone(), action_id.clone())).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    for template_id in templates {
        WORKFLOW_ACTION_TEMPLATES.remove(storage, (workflow_id.clone(), action_id.clone(), template_id.clone()));
    }
    Ok(())
}

pub fn save_workflow_action_contracts(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId, contracts: &HashSet<String>) -> StdResult<()> {
    for contract in contracts {
        WORKFLOW_ACTION_CONTRACTS.save(storage, (workflow_id.clone(), action_id.clone(), contract.clone()), &())?;
    }
    Ok(())
}

pub fn load_workflow_action_contract(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId, contract_addr: &str) -> StdResult<()> {
    WORKFLOW_ACTION_CONTRACTS.load(storage, (workflow_id.clone(), action_id.clone(), contract_addr.to_string()))
}

pub fn load_workflow_action_contracts(storage: &dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<HashSet<String>> {
    let contracts = WORKFLOW_ACTION_CONTRACTS.prefix((workflow_id.clone(), action_id.clone())).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    Ok(contracts.into_iter().collect())
}

pub fn remove_workflow_action_contracts(storage: &mut dyn Storage, workflow_id: &WorkflowId, action_id: &ActionId) -> StdResult<()> {
    let contracts = WORKFLOW_ACTION_CONTRACTS.prefix((workflow_id.clone(), action_id.clone())).keys(storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    for contract in contracts {
        WORKFLOW_ACTION_CONTRACTS.remove(storage, (workflow_id.clone(), action_id.clone(), contract.clone()));
    }
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
// ========== CONFIG =============
// =============================== 

pub const CONFIG: Item<Config> = Item::new("conf");

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn validate_sender_is_publisher(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), ContractError> {
    let state = load_config(storage)?;
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
    let state = load_config(storage)?;
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
    let state = load_config(storage)?;
    if info.sender != state.owner {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_contract_is_whitelisted(
    storage: &dyn Storage,
    workflow_id: &WorkflowId,
    action_id: &ActionId,
    contract_addr: &str,
) -> Result<(), ContractError> {
    let _ = load_workflow_action_contract(storage, workflow_id, action_id, contract_addr).map_err(|_| {
        ContractError::ContractNotWhitelisted {
            contract: contract_addr.to_string(),
            workflow_id: workflow_id.to_string(),
        }
    })?;
    Ok(())
}
