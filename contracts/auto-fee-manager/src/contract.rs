#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::*;
use crate::helpers::validate_address;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg, MigrateMsg};
use crate::state::{ACCEPTED_DENOMS, CONFIG, Config};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, crate::CONTRACT_NAME, crate::CONTRACT_VERSION)?;

    // Validate execution_fees_destination_address is not empty and is a valid address
    validate_address(&deps, &msg.execution_fees_destination_address.as_str(), "execution_fees_destination_address")?;

    // Validate distribution_fees_destination_address is not empty and is a valid address
    validate_address(&deps, &msg.distribution_fees_destination_address.as_str(), "distribution_fees_destination_address")?;

    // Validate authorized_address is not empty and is a valid address
    validate_address(&deps, &msg.crank_authorized_address.as_str(), "authorized_address")?;

    // if workflow_manager_address is not empty, validate it is a valid address
    if let Some(workflow_manager_address) = msg.workflow_manager_address.clone() {
        validate_address(&deps, &workflow_manager_address.as_str(), "workflow_manager_address")?;
    }

    // Validate accepted_denoms is not empty
    if msg.accepted_denoms.is_empty() {
        return Err(ContractError::EmptyAcceptedDenoms {});
    }

    // Initialize ACCEPTED_DENOMS
    ACCEPTED_DENOMS.save(deps.storage, &msg.accepted_denoms)?;

    let config = Config {
        execution_fees_destination_address: msg.execution_fees_destination_address,
        distribution_fees_destination_address: msg.distribution_fees_destination_address,
        crank_authorized_address: msg.crank_authorized_address,
        workflow_manager_address: msg.workflow_manager_address,
        creator_distribution_fee: msg.creator_distribution_fee,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => handle_deposit(deps, info),
        ExecuteMsg::Withdraw { denom, amount } => handle_withdraw(deps, info, denom, amount),
        ExecuteMsg::ChargeFeesFromUserBalance { batch } => {
            handle_charge_fees_from_user_balance(deps, env, info, batch)
        }
        ExecuteMsg::ChargeFeesFromMessageCoins {
            fees,
        } => handle_charge_fees_from_message_coins(deps, env, info, fees),
        ExecuteMsg::ClaimCreatorFees {} => handle_claim_creator_fees(deps, info),
        ExecuteMsg::DistributeNonCreatorFees {} => {
            handle_distribute_non_creator_fees(deps, env, info)
        }
        ExecuteMsg::DistributeCreatorFees {} => {
            handle_distribute_creator_fees(deps, env, info)
        }
        ExecuteMsg::EnableCreatorFeeDistribution {} => {
            handle_enable_creator_fee_distribution(deps, info)
        }
        ExecuteMsg::DisableCreatorFeeDistribution {} => {
            handle_disable_creator_fee_distribution(deps, info)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HasExceededDebtLimit { user } => {
            let result = has_exceeded_debt_limit(deps, user)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetUserBalances { user } => {
            let result = get_user_balances(deps, user)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetCreatorFees { creator } => {
            let result = get_creator_fees(deps, creator)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetNonCreatorFees {} => {
            let result = get_non_creator_fees(deps)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::IsCreatorSubscribed { creator } => {
            let result = is_creator_subscribed(deps, creator)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetSubscribedCreators {} => {
            let result = get_subscribed_creators(deps)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetConfig {} => {
            let config = CONFIG.load(deps.storage)?;
            let accepted_denoms = ACCEPTED_DENOMS.load(deps.storage)?;
            let result = InstantiateMsg {
                accepted_denoms: accepted_denoms,
                execution_fees_destination_address: config.execution_fees_destination_address,
                distribution_fees_destination_address: config.distribution_fees_destination_address,
                crank_authorized_address: config.crank_authorized_address,
                workflow_manager_address: config.workflow_manager_address,
                creator_distribution_fee: config.creator_distribution_fee,
            };
            cosmwasm_std::to_json_binary(&result)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::SetCrankAuthorizedAddress { address } => {
            validate_address(&deps, &address.as_str(), "authorized_address")?;
            let mut config = CONFIG.load(deps.storage)?;
            config.crank_authorized_address = address;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(
                cosmwasm_std::Event::new("autorujira-fee-manager/sudo_set_crank_authorized_address")
            ))
        }
        SudoMsg::SetWorkflowManagerAddress { address } => {
            validate_address(&deps, &address.as_str(), "workflow_manager_address")?;
            let mut config = CONFIG.load(deps.storage)?;
            config.workflow_manager_address = Some(address);
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(
                cosmwasm_std::Event::new("autorujira-fee-manager/sudo_set_workflow_manager_address")
            ))
        }
        SudoMsg::SetExecutionFeesDestinationAddress { address } => {
            validate_address(&deps, &address.as_str(), "execution_fees_destination_address")?;
            let mut config = CONFIG.load(deps.storage)?;
            config.execution_fees_destination_address = address;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(
                cosmwasm_std::Event::new("autorujira-fee-manager/sudo_set_execution_fees_destination_address")
            ))
        }
        SudoMsg::SetDistributionFeesDestinationAddress { address } => {
            validate_address(&deps, &address.as_str(), "distribution_fees_destination_address")?;
            let mut config = CONFIG.load(deps.storage)?;
            config.distribution_fees_destination_address = address;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(
                cosmwasm_std::Event::new("autorujira-fee-manager/sudo_set_distribution_fees_destination_address")
            ))
        }
        SudoMsg::SetCreatorDistributionFee { fee } => {
            let mut config = CONFIG.load(deps.storage)?;
            config.creator_distribution_fee = fee;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(
                cosmwasm_std::Event::new("autorujira-fee-manager/sudo_set_creator_distribution_fee")
            ))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Update contract version
    set_contract_version(deps.storage, crate::CONTRACT_NAME, crate::CONTRACT_VERSION)?;
    
    // No migration logic needed for this version
    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/migrate")
                .add_attribute("version", crate::CONTRACT_VERSION)
        )
    )
}
