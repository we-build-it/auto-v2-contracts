#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Reply,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    execute::{
        cancel_run, cancel_schedule, charge_fees, execute_action, execute_instance, pause_schedule,
        publish_workflow, remove_user_payment_config_execute, resume_schedule,
        set_user_payment_config,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    query::{
        query_instances_by_requester, query_user_payment_config, query_workflow_by_id,
        query_workflow_instance,
    },
    state::{load_config, save_config, Config},
    utils::build_authz_execute_contract_msg,
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
    let state = Config {
        owner: info.sender.clone(),
        allowed_publishers: msg.allowed_publishers,
        allowed_action_executors: msg.allowed_action_executors,
        referral_memo: msg.referral_memo,
        fee_manager_address: msg.fee_manager_address,
        allowance_denom: msg.allowance_denom,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    save_config(deps.storage, &state)?;

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
        ExecuteMsg::CancelRun {
            instance_id,
            run_id,
        } => cancel_run(deps, env, info, instance_id, run_id),
        ExecuteMsg::CancelSchedule { instance_id } => cancel_schedule(deps, env, info, instance_id),
        ExecuteMsg::PauseSchedule { instance_id } => pause_schedule(deps, env, info, instance_id),
        ExecuteMsg::ResumeSchedule { instance_id } => resume_schedule(deps, env, info, instance_id),
        ExecuteMsg::ExecuteAction {
            user_address,
            instance_id,
            action_id,
            template_id,
            params,
        } => execute_action(
            deps,
            env,
            info,
            user_address,
            instance_id,
            action_id,
            template_id,
            params,
        ),
        ExecuteMsg::SetUserPaymentConfig {
            user_address,
            payment_config,
        } => set_user_payment_config(deps, env, info, user_address, payment_config),
        ExecuteMsg::RemoveUserPaymentConfig { user_address } => {
            remove_user_payment_config_execute(deps, env, info, user_address)
        }
        ExecuteMsg::ChargeFees { batch_id, fees } => charge_fees(deps, env, info, batch_id, fees),
        // TODO: temporal AuthZ test, remove this
        ExecuteMsg::TestAuthz { } => {
            let daodao_msg = "{ \"echo\": { \"message\": \"T3BlcmFjaW9uIGRlIFN0YWtl\", \"attributes\": [[\"priority\", \"high\"],[\"timestamp\", \"1640995200\"]] } }";
            let authz_msg = build_authz_execute_contract_msg(
                &env, 
                &info.sender, 
                &deps.api.addr_validate("tthor14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sw58u9f").unwrap(), 
                &daodao_msg.to_string(), 
                &vec![])
                .unwrap();
            Ok(Response::new().add_message(authz_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    // Handle replies from submessages
    match reply.id {
        // Handle fee manager replies (instance_id is used as reply_id)
        id if id > 0 => {
            // This is a fee manager reply
            crate::execute::handle_fee_manager_reply(deps, env, reply)
        }
        _ => Err(ContractError::GenericError("Unknown reply ID".to_string())),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;
    match msg {
        SudoMsg::SetOwner(owner) => {
            config.owner = owner;
        }
        SudoMsg::SetAllowedPublishers(allowed_publishers) => {
            config.allowed_publishers = allowed_publishers;
        }
        SudoMsg::SetAllowedActionExecutors(allowed_action_executors) => {
            config.allowed_action_executors = allowed_action_executors;
        }
        SudoMsg::SetReferralMemo(referral_memo) => {
            config.referral_memo = referral_memo;
        }
    }
    save_config(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: ()) -> StdResult<Response> {
    // Update version if changed
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Migrate HERE other parts of state when needed
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInstancesByRequester { requester_address } => {
            to_json_binary(&query_instances_by_requester(deps, requester_address)?)
        }
        QueryMsg::GetWorkflowById { workflow_id } => {
            to_json_binary(&query_workflow_by_id(deps, workflow_id)?)
        }
        QueryMsg::GetWorkflowInstance {
            user_address,
            instance_id,
        } => to_json_binary(&query_workflow_instance(deps, user_address, instance_id)?),
        QueryMsg::GetUserPaymentConfig { user_address } => {
            to_json_binary(&query_user_payment_config(deps, user_address)?)
        }
        QueryMsg::GetConfig {} => {
            let config = load_config(deps.storage)?;
            let result = InstantiateMsg {
                allowed_publishers: config.allowed_publishers.clone(),
                allowed_action_executors: config.allowed_action_executors.clone(),
                referral_memo: config.referral_memo,
                fee_manager_address: config.fee_manager_address,
                allowance_denom: config.allowance_denom,
            };
            to_json_binary(&result)
        }
    }
}
