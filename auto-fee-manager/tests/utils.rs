use cosmwasm_std::{
    testing::message_info,
    Addr, DepsMut, Env, Response, Coin, Uint128,
};

use auto_fee_manager::{
    contract::{execute, instantiate, sudo},
    msg::{ExecuteMsg, InstantiateMsg, SudoMsg, UserFees, Fee, FeeType},
    ContractError,
};

/// Initialize the contract with the given parameters
pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    admin: Addr,
    max_debt: Coin,
    min_balance_threshold: Coin,
    gas_destination_address: Addr,
    infra_destination_address: Addr,
    accepted_denoms: Vec<String>,
    crank_authorized_address: Addr,
    workflow_manager_address: Addr,
) -> Result<Response, ContractError> {
    let instantiate_msg = InstantiateMsg {
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address,
        workflow_manager_address,
    };
    let instantiate_info = message_info(&admin, &[]);
    instantiate(deps, env, instantiate_info, instantiate_msg)
}

/// Create a test UserFees struct
pub fn create_test_user_fees(user: Addr) -> UserFees {
    let user_clone = user.clone();
    UserFees {
        user,
        fees: vec![
            Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test execution fee".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(1000u128),
                denom: "uusdc".to_string(),
                fee_type: FeeType::Execution,
                creator_address: None,
            },
            Fee {
                workflow_instance_id: "test-instance-2".to_string(),
                action_id: "test-action-2".to_string(),
                description: "Test creator fee".to_string(),
                timestamp: 1234567891,
                amount: Uint128::from(2000u128),
                denom: "uusdc".to_string(),
                fee_type: FeeType::Creator,
                creator_address: Some(user_clone),
            },
        ],
    }
}

/// Execute ChargeFeesFromUserBalance
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
pub fn execute_charge_fees_from_message_coins(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    fees: Vec<Fee>,
    creator_address: Addr,
) -> Result<Response, ContractError> {
    let execute_msg = ExecuteMsg::ChargeFeesFromMessageCoins {
        fees,
        creator_address,
    };
    let execute_info = message_info(&sender, &[]);
    execute(deps, env, execute_info, execute_msg)
}

/// Execute DistributeCreatorFees
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
pub fn sudo_set_crank_authorized_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetCrankAuthorizedAddress { address };
    sudo(deps, env, sudo_msg)
}

/// Execute SetWorkflowManagerAddress sudo
pub fn sudo_set_workflow_manager_address(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let sudo_msg = SudoMsg::SetWorkflowManagerAddress { address };
    sudo(deps, env, sudo_msg)
} 