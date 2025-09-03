use cosmwasm_std::{
    testing::message_info,
    Addr, DepsMut, Env, Response, Coin, Uint128,
};

use auto_fee_manager::{
    contract::{execute, instantiate, sudo},
    msg::{ExecuteMsg, InstantiateMsg, SudoMsg, UserFees, Fee, FeeType},
    ContractError,
};

use std::collections::HashMap;

/// Initialize the contract with the given parameters
pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    admin: Addr,
    max_debt: Coin,
    min_balance_threshold: Coin,
    execution_fees_destination_address: Addr,
    distribution_fees_destination_address: Addr,
    accepted_denoms: Vec<String>,
    crank_authorized_address: Addr,
    workflow_manager_address: Addr,
    creator_distribution_fee: Uint128,
) -> Result<Response, ContractError> {
    let instantiate_msg = InstantiateMsg {
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address,
        workflow_manager_address,
        creator_distribution_fee,
    };
    let instantiate_info = message_info(&admin, &[]);
    instantiate(deps, env, instantiate_info, instantiate_msg)
}

/// Create a test UserFees struct
#[allow(dead_code)]
pub fn create_test_user_fees(user: Addr) -> UserFees {
    let user_clone = user.clone();
    UserFees {
        user,
        fees: vec![
            Fee {
                timestamp: 1234567890,
                amount: Uint128::from(1000u128),
                denom: "uusdc".to_string(),
                fee_type: FeeType::Execution,
                creator_address: None,
            },
            Fee {
                timestamp: 1234567891,
                amount: Uint128::from(2000u128),
                denom: "uusdc".to_string(),
                fee_type: FeeType::Creator,
                creator_address: Some(user_clone),
            },
        ],
    }
}

/// Create a test UserFees struct with specific creator and denom
#[allow(dead_code)]
pub fn create_test_user_fees_with_creator(
    user: Addr,
    creator: Addr,
    denom: String,
    amount: Uint128,
) -> UserFees {
    UserFees {
        user,
        fees: vec![
            Fee {
                timestamp: 1234567890,
                amount,
                denom,
                fee_type: FeeType::Creator,
                creator_address: Some(creator),
            },
        ],
    }
}

/// Execute ChargeFeesFromUserBalance
#[allow(dead_code)]
pub fn execute_charge_fees_from_user_balance(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    batch: Vec<UserFees>,
) -> Result<Response, ContractError> {
    let execute_msg = ExecuteMsg::ChargeFeesFromUserBalance { batch };
    let execute_info = message_info(&sender, &[]);
    execute(deps, env, execute_info, execute_msg)
}

/// Execute ChargeFeesFromMessageCoins
#[allow(dead_code)]
pub fn execute_charge_fees_from_message_coins(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    fees: Vec<Fee>,
) -> Result<Response, ContractError> {
    // Calculate expected funds from fees
    let mut expected_funds: Vec<Coin> = Vec::new();
    let mut by_denom: HashMap<String, Uint128> = HashMap::new();
    for fee in &fees {
        *by_denom.entry(fee.denom.clone()).or_insert(Uint128::zero()) += fee.amount;
    }
    for (denom, amount) in by_denom {
        expected_funds.push(Coin { denom, amount });
    }
    let execute_msg = ExecuteMsg::ChargeFeesFromMessageCoins {
        fees,
    };
    let execute_info = message_info(&sender, &expected_funds);
    execute(deps, env, execute_info, execute_msg)
}

/// Execute DistributeCreatorFees
#[allow(dead_code)]
pub fn execute_distribute_creator_fees(
    deps: DepsMut,
    env: Env,
    sender: Addr,
) -> Result<Response, ContractError> {
    let execute_msg = ExecuteMsg::DistributeCreatorFees {};
    let execute_info = message_info(&sender, &[]);
    execute(deps, env, execute_info, execute_msg)
}

/// Execute DistributeNonCreatorFees
#[allow(dead_code)]
pub fn execute_distribute_non_creator_fees(
    deps: DepsMut,
    env: Env,
    sender: Addr,
) -> Result<Response, ContractError> {
    let execute_msg = ExecuteMsg::DistributeNonCreatorFees {};
    let execute_info = message_info(&sender, &[]);
    execute(deps, env, execute_info, execute_msg)
}

/// Execute SetCrankAuthorizedAddress sudo
#[allow(dead_code)]
pub fn sudo_set_crank_authorized_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetCrankAuthorizedAddress { address };
    sudo(deps, env, sudo_msg)
}

/// Execute SetWorkflowManagerAddress sudo
#[allow(dead_code)]
pub fn sudo_set_workflow_manager_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetWorkflowManagerAddress { address };
    sudo(deps, env, sudo_msg)
}

/// Execute SetExecutionFeesDestinationAddress sudo
#[allow(dead_code)]
pub fn sudo_set_execution_fees_destination_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetExecutionFeesDestinationAddress { address };
    sudo(deps, env, sudo_msg)
}

/// Execute SetDistributionFeesDestinationAddress sudo
#[allow(dead_code)]
pub fn sudo_set_distribution_fees_destination_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetDistributionFeesDestinationAddress { address };
    sudo(deps, env, sudo_msg)
}

#[allow(dead_code)]
pub fn _sudo_set_creator_distribution_fee(
    deps: DepsMut,
    env: Env,
    _admin: Addr,
    fee: Uint128,
) -> Result<Response, ContractError> {
    let sudo_msg = auto_fee_manager::msg::SudoMsg::SetCreatorDistributionFee { fee };
    sudo(deps, env, sudo_msg)
}

#[allow(dead_code)]
pub fn execute_claim_creator_fees(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
) -> Result<Response, ContractError> {
    let info = message_info(&sender, &[]);
    auto_fee_manager::handlers::handle_claim_creator_fees(deps, info)
} 