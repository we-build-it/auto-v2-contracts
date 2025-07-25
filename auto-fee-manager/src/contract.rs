#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::*;
use crate::helpers::validate_address;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{ACCEPTED_DENOMS, CONFIG, Config};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, crate::CONTRACT_NAME, crate::CONTRACT_VERSION)?;

    // Validate max_debt is not negative (can be zero)
    if msg.max_debt.amount < Uint128::zero() {
        return Err(ContractError::InvalidMaxDebt {
            reason: "max_debt cannot be negative".to_string(),
        });
    }

    // Validate min_balance_threshold is not negative (can be zero)
    if msg.min_balance_threshold.amount < Uint128::zero() {
        return Err(ContractError::InvalidMaxDebt {
            reason: "min_balance_threshold cannot be negative".to_string(),
        });
    }

    // Validate gas_destination_address is not empty and is a valid address
    validate_address(&deps, &msg.gas_destination_address.as_str(), "gas_destination_address")?;

    // Validate infra_destination_address is not empty and is a valid address
    validate_address(&deps, &msg.infra_destination_address.as_str(), "infra_destination_address")?;

    // Validate authorized_address is not empty and is a valid address
    validate_address(&deps, &msg.crank_authorized_address.as_str(), "authorized_address")?;

    // Validate workflow_manager_address is not empty and is a valid address
    validate_address(&deps, &msg.workflow_manager_address.as_str(), "workflow_manager_address")?;

    // Validate accepted_denoms is not empty
    if msg.accepted_denoms.is_empty() {
        return Err(ContractError::EmptyAcceptedDenoms {});
    }

    // Initialize ACCEPTED_DENOMS
    ACCEPTED_DENOMS.save(deps.storage, &msg.accepted_denoms)?;

    let config = Config {
        max_debt: msg.max_debt,
        min_balance_threshold: msg.min_balance_threshold,
        gas_destination_address: msg.gas_destination_address,
        infra_destination_address: msg.infra_destination_address,
        crank_authorized_address: msg.crank_authorized_address,
        workflow_manager_address: msg.workflow_manager_address,
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
            creator_address,
        } => handle_charge_fees_from_message_coins(deps, env, info, fees, creator_address),
        ExecuteMsg::ClaimCreatorFees {} => handle_claim_creator_fees(deps, info),
        ExecuteMsg::DistributeNonCreatorFees {} => {
            handle_distribute_non_creator_fees(deps, env, info)
        }
        ExecuteMsg::DistributeCreatorFees {} => {
            handle_distribute_creator_fees(deps, env, info)
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::SetMaxDebt { denom: _, amount: _ } => {
            // TODO: Implement SetMaxDebt logic
            Ok(Response::new())
        }
        SudoMsg::SetReceiverAddress { fee_type: _, address: _ } => {
            // TODO: Implement SetReceiverAddress logic
            Ok(Response::new())
        }
        SudoMsg::SetCrankAuthorizedAddress { address } => {
            // Validate address is not empty
            validate_address(&deps, &address.as_str(), "authorized_address")?;
            
            // Load current config
            let mut config = CONFIG.load(deps.storage)?;
            
            // Update authorized_address
            config.crank_authorized_address = address;
            
            // Save updated config
            CONFIG.save(deps.storage, &config)?;
            
            Ok(Response::new())
        }
        SudoMsg::SetWorkflowManagerAddress { address } => {
            // Validate address is not empty
            validate_address(&deps, &address.as_str(), "workflow_manager_address")?;
            
            // Load current config
            let mut config = CONFIG.load(deps.storage)?;
            
            // Update workflow_manager_address
            config.workflow_manager_address = address;
            
            // Save updated config
            CONFIG.save(deps.storage, &config)?;
            
            Ok(Response::new())
        }
    }
}
