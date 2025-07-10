#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{save_ownership, Ownership},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:proxy-auth";
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
        approvers: msg.approvers,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    save_ownership(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("approvers_count", state.approvers.len().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RequestForApproval { template } => {
            crate::execute::request_for_approval(deps, env, info, template)
        }
        ExecuteMsg::ApproveTemplate { template_id } => {
            crate::execute::approve_template(deps, env, info, template_id)
        }
        ExecuteMsg::RejectTemplate { template_id } => {
            crate::execute::reject_template(deps, env, info, template_id)
        }
        ExecuteMsg::ExecuteFlow {
            flow_id,
            template_id,
            params,
        } => crate::execute::execute_flow(deps, env, info, flow_id, template_id, params),
        ExecuteMsg::CancelFlow { flow_id } => crate::execute::cancel_flow(deps, env, info, flow_id),
        ExecuteMsg::ExecuteAction {
            flow_id,
            action_id,
            params,
            funds,
        } => crate::execute::execute_action(deps, env, info, flow_id, action_id, params, funds),
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
        QueryMsg::GetFlowsByRequester { requester_address } => {
            to_json_binary(&crate::query::query_flows_by_requester(deps, requester_address)?)
        }
        QueryMsg::GetTemplatesByPublisher { publisher_address } => to_json_binary(
            &crate::query::query_templates_by_publisher(deps, publisher_address)?,
        ),
        QueryMsg::GetFlowById { flow_id } => {
            to_json_binary(&crate::query::query_flow_by_id(deps, flow_id)?)
        }
        QueryMsg::GetTemplateById { template_id } => {
            to_json_binary(&crate::query::query_template_by_id(deps, template_id)?)
        }
    }
}
