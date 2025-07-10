#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TemplateMsg},
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
            execute::request_for_approval(deps, env, info, template)
        }
        ExecuteMsg::ApproveTemplate { template_id } => {
            execute::approve_template(deps, env, info, template_id)
        }
        ExecuteMsg::RejectTemplate { template_id } => {
            execute::reject_template(deps, env, info, template_id)
        }
        ExecuteMsg::ExecuteFlow {
            flow_id,
            template_id,
            params,
        } => execute::execute_flow(deps, env, info, flow_id, template_id, params),
        ExecuteMsg::CancelFlow { flow_id } => execute::cancel_flow(deps, env, info, flow_id),
        ExecuteMsg::ExecuteAction {
            flow_id,
            action_id,
            params,
            funds,
        } => execute::execute_action(deps, env, info, flow_id, action_id, params, funds),
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
            to_json_binary(&query::query_flows_by_requester(deps, requester_address)?)
        }
        QueryMsg::GetTemplatesByPublisher { publisher_address } => to_json_binary(
            &query::query_templates_by_publisher(deps, publisher_address)?,
        ),
        QueryMsg::GetFlowById { flow_id } => {
            to_json_binary(&query::query_flow_by_id(deps, flow_id)?)
        }
        QueryMsg::GetTemplateById { template_id } => {
            to_json_binary(&query::query_template_by_id(deps, template_id)?)
        }
    }
}

pub mod execute {
    use crate::{utils::{render_template, build_authz_msg, AuthzMessageType}, state::{
        load_flow, load_template, load_template_action, remove_template, save_flow, save_template,
        save_template_action, validate_sender_is_admin, validate_sender_is_approver, Action, Flow,
        Template,
    }};
    use cosmwasm_std::Coin;
    use std::collections::HashMap;

    use super::*;

    pub fn request_for_approval(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        template: TemplateMsg,
    ) -> Result<Response, ContractError> {
        // Check if template already exists
        if load_template(deps.storage, &template.id).is_ok() {
            return Err(ContractError::TemplateAlreadyExists {
                template_id: template.id.clone(),
            });
        }

        let new_template = Template {
            id: template.id.clone(),
            publisher: info.sender.clone(),
            approved: false,
            private: template.private,
        };

        save_template(deps.storage, &new_template)?;

        // Iterate and save template actions
        for action_msg in template.actions {
            let action = Action {
                id: action_msg.id,
                template_id: template.id.clone(),
                message_template: action_msg.message_template,
                target_contract: action_msg.contract_address,
                allowed_denoms: action_msg.allowed_denoms,
            };

            save_template_action(deps.storage, &template.id, &action.id, &action)?;
        }

        Ok(Response::new()
            .add_attribute("method", "request_for_approval")
            .add_attribute("template_id", template.id)
            .add_attribute("publisher", info.sender.to_string()))
    }

    pub fn approve_template(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        template_id: String,
    ) -> Result<Response, ContractError> {
        // Validate that sender is an approver
        validate_sender_is_approver(deps.storage, &info)?;

        // Load the template
        let mut template = load_template(deps.storage, &template_id)?;

        // Check that it's not already approved
        if template.approved {
            return Err(ContractError::TemplateAlreadyApproved {
                template_id: template_id.clone(),
            });
        }

        // Mark as approved
        template.approved = true;

        // Save the updated template
        save_template(deps.storage, &template)?;

        Ok(Response::new()
            .add_attribute("method", "approve_template")
            .add_attribute("template_id", template_id)
            .add_attribute("approver", info.sender.to_string()))
    }

    pub fn reject_template(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        template_id: String,
    ) -> Result<Response, ContractError> {
        // Validate that sender is an approver
        validate_sender_is_approver(deps.storage, &info)?;

        // Check that template exists
        let template = load_template(deps.storage, &template_id)?;

        // Check that it's not already approved
        if template.approved {
            return Err(ContractError::TemplateAlreadyApproved {
                template_id: template_id.clone(),
            });
        }

        // Remove template and all its actions
        remove_template(deps.storage, &template_id)?;

        Ok(Response::new()
            .add_attribute("method", "reject_template")
            .add_attribute("template_id", template_id)
            .add_attribute("rejecter", info.sender.to_string()))
    }

    pub fn execute_flow(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        flow_id: String,
        template_id: String,
        params: String,
    ) -> Result<Response, ContractError> {
        // Check if flow already exists
        if load_flow(deps.storage, &flow_id).is_ok() {
            return Err(ContractError::FlowAlreadyExists {
                flow_id: flow_id.clone(),
            });
        }

        // Load and validate template exists
        let template = load_template(deps.storage, &template_id).map_err(|_| {
            ContractError::TemplateNotFound {
                template_id: template_id.clone(),
            }
        })?;

        // Check if template is approved
        if !template.approved {
            return Err(ContractError::TemplateNotApproved {
                template_id: template_id.clone(),
            });
        }

        // Check if template is private and sender is not the publisher
        if template.private && info.sender != template.publisher {
            return Err(ContractError::TemplatePrivateAccessDenied {
                template_id: template_id.clone(),
            });
        }

        // Create new flow
        let new_flow = Flow {
            id: flow_id.clone(),
            template_id: template_id.clone(),
            params,
            requester: info.sender.clone(),
        };

        // Save the flow
        save_flow(deps.storage, &new_flow)?;

        Ok(Response::new()
            .add_attribute("method", "execute_flow")
            .add_attribute("flow_id", flow_id)
            .add_attribute("template_id", template_id)
            .add_attribute("requester", info.sender.to_string()))
    }

    pub fn cancel_flow(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        flow_id: String,
    ) -> Result<Response, ContractError> {
        // Load the flow
        let flow = crate::state::load_flow(deps.storage, &flow_id).map_err(|_| {
            ContractError::FlowNotFound {
                flow_id: flow_id.clone(),
            }
        })?;

        // Validate that the requester is the sender
        if flow.requester != info.sender {
            return Err(ContractError::FlowCancelUnauthorized { flow_id });
        }

        // Remove the flow
        crate::state::remove_flow(deps.storage, &flow_id)?;

        Ok(Response::new()
            .add_attribute("method", "cancel_flow")
            .add_attribute("flow_id", flow_id)
            .add_attribute("canceller", info.sender.to_string()))
    }

    pub fn execute_action(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        flow_id: String,
        action_id: String,
        params: Option<HashMap<String, String>>,
        funds: Option<Vec<Coin>>,
    ) -> Result<Response, ContractError> {
        // Validate that sender is admin
        validate_sender_is_admin(deps.storage, &info)?;

        // 1) Validate that the flow exists
        let flow = load_flow(deps.storage, &flow_id).map_err(|_| ContractError::FlowNotFound {
            flow_id: flow_id.clone(),
        })?;

        // Validate that the template exists
        load_template(deps.storage, &flow.template_id).map_err(|_| {
            ContractError::TemplateNotFound {
                template_id: flow.template_id.clone(),
            }
        })?;

        // Validate that the action exists in the template
        let action =
            load_template_action(deps.storage, &flow.template_id, &action_id).map_err(|_| {
                ContractError::ActionNotFound {
                    template_id: flow.template_id.clone(),
                    action_id: action_id.clone(),
                }
            })?;

        // Validate denoms if funds are provided
        if let Some(funds_vec) = &funds {
            for coin in funds_vec {
                if !action.allowed_denoms.contains(&coin.denom) {
                    return Err(ContractError::InvalidDenom(coin.denom.clone()));
                }
            }
        }

        // Handle optional params
        let params_map = params.unwrap_or_default();
        
        // Replace params in message template
        let rendered = render_template(&action.message_template, &params_map)?;
        // let value: serde_json::Value = serde_json::from_str(&rendered)
        //     .map_err(|e| ContractError::GenericError(format!("JSON parsing error: {}", e)))?;
        // let binary_msg: Binary = to_json_binary(&value)?;

        // build authz message in name of the user (flow.requester)
        let authz_msg = build_authz_msg(
            env,
            flow.requester.clone(),
            AuthzMessageType::ExecuteContract {
                contract_addr: action.target_contract.clone(),
                msg_str: rendered,
                funds: funds.unwrap_or_default(),
            },
        )?;

        Ok(Response::new()
            .add_message(authz_msg)
            .add_attribute("method", "execute_action")
            .add_attribute("flow_id", flow_id)
            .add_attribute("action_id", action_id)
            .add_attribute("executor", info.sender.to_string()))
    }
}

pub mod query {
    use super::*;
    use crate::{
        msg::{FlowResponse, FlowsResponse, TemplateResponse, TemplatesResponse},
        state::{load_flow, load_template},
    };

    pub fn query_flows_by_requester(
        deps: Deps,
        requester_address: String,
    ) -> StdResult<FlowsResponse> {
        let requester = deps.api.addr_validate(&requester_address)?;
        let flows = crate::state::query_flows_by_requester(deps.storage, requester)?;
        Ok(FlowsResponse { flows })
    }

    pub fn query_templates_by_publisher(
        deps: Deps,
        publisher_address: String,
    ) -> StdResult<TemplatesResponse> {
        let publisher = deps.api.addr_validate(&publisher_address)?;
        let templates = crate::state::query_templates_by_publisher(deps.storage, publisher)?;
        Ok(TemplatesResponse { templates })
    }

    pub fn query_flow_by_id(deps: Deps, flow_id: String) -> StdResult<FlowResponse> {
        let flow = load_flow(deps.storage, &flow_id)?;
        Ok(FlowResponse { flow })
    }

    pub fn query_template_by_id(deps: Deps, template_id: String) -> StdResult<TemplateResponse> {
        let template = load_template(deps.storage, &template_id)?;
        Ok(TemplateResponse { template })
    }
}
