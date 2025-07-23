#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    execute::{
        cancel_instance, execute_instance, pause_instance, publish_workflow, resume_instance,
    },
    execute::execute_action,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{query_instances_by_requester, query_workflow_by_id, query_workflow_instance},
    state::{save_ownership, Ownership},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:workflow-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn validate_no_funds_received(info: &MessageInfo) -> Result<(), ContractError> {
    if !info.funds.is_empty() {
        Err(ContractError::InvalidFundsReceived {})
    } else {
        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Ownership {
        owner: info.sender.clone(),
        allowed_publishers: msg.allowed_publishers,
        allowed_action_executors: msg.allowed_action_executors,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    save_ownership(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute(
            "approvers_count",
            state.allowed_publishers.len().to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PublishWorkflow { workflow } => publish_workflow(deps, env, info, workflow),
        ExecuteMsg::ExecuteInstance { instance } => execute_instance(deps, env, info, instance),
        ExecuteMsg::CancelInstance { instance_id } => cancel_instance(deps, env, info, instance_id),
        ExecuteMsg::PauselInstance { instance_id } => pause_instance(deps, env, info, instance_id),
        ExecuteMsg::ResumeInstance { instance_id } => resume_instance(deps, env, info, instance_id),
        ExecuteMsg::ExecuteAction {
            user_address,
            instance_id,
            action_id,
            params,
        } => execute_action(
            deps,
            env,
            info,
            user_address,
            instance_id,
            action_id,
            params,
        ),
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
//     // Update version if changed
//     set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
//     // Migrate HERE other parts of state when needed
//     Ok(Response::default())
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInstancesByRequester { requester_address } => {
            to_json_binary(&query_instances_by_requester(deps, requester_address)?)
        }
        QueryMsg::GetWorkflowById { template_id } => {
            to_json_binary(&query_workflow_by_id(deps, template_id)?)
        }
        QueryMsg::GetWorkflowInstance { user_address, instance_id } => {
            to_json_binary(&query_workflow_instance(deps, user_address, instance_id)?)
        }
    }
}
