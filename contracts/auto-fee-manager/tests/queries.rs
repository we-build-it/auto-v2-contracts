use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use cosmwasm_std::{Coin, Uint128};

mod utils;
use crate::utils::*;

use auto_fee_manager::msg::{UserFees, Fee, FeeType};

#[test]
fn test_get_creator_fees_with_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let creator_address = api.addr_make("creator");
    
    let max_debt = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["tcy".to_string()];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();

    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    }];
    let deposit_info = cosmwasm_std::testing::message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();

    // Create test fees with Creator type
    let test_fees = create_test_user_fees_with_creator(
        user_address.clone(),
        creator_address.clone(),
        "tcy".to_string(),
        Uint128::from(100u128),
    );

    // Charge fees from user balance
    execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        vec![test_fees],
    ).unwrap();

    // Query creator fees
    let query_msg = auto_fee_manager::msg::QueryMsg::GetCreatorFees {
        creator: creator_address.clone(),
    };
    let result = auto_fee_manager::contract::query(deps.as_ref(), env, query_msg).unwrap();
    let response: auto_fee_manager::msg::CreatorFeesResponse = cosmwasm_std::from_json(result).unwrap();

    // Verify response
    assert_eq!(response.creator, creator_address);
    assert_eq!(response.fees.len(), 1);
    assert_eq!(response.fees[0].denom, "tcy");
    assert_eq!(response.fees[0].balance, Uint128::from(100u128));
}

#[test]
fn test_get_creator_fees_without_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let creator_address = api.addr_make("creator");
    
    let max_debt = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["tcy".to_string()];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();

    // Query creator fees for a creator that has no fees
    let query_msg = auto_fee_manager::msg::QueryMsg::GetCreatorFees {
        creator: creator_address.clone(),
    };
    let result = auto_fee_manager::contract::query(deps.as_ref(), env, query_msg).unwrap();
    let response: auto_fee_manager::msg::CreatorFeesResponse = cosmwasm_std::from_json(result).unwrap();

    // Verify response - should have no fees
    assert_eq!(response.creator, creator_address);
    assert_eq!(response.fees.len(), 0);
}

#[test]
fn test_get_user_balances() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["tcy".to_string()];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();

    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(500u128),
    }];
    let deposit_info = cosmwasm_std::testing::message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();

    // Query user balances
    let query_msg = auto_fee_manager::msg::QueryMsg::GetUserBalances {
        user: user_address.clone(),
    };
    let result = auto_fee_manager::contract::query(deps.as_ref(), env, query_msg).unwrap();
    let response: auto_fee_manager::msg::UserBalancesResponse = cosmwasm_std::from_json(result).unwrap();

    // Verify response
    assert_eq!(response.user, user_address);
    assert_eq!(response.balances.len(), 1);
    assert_eq!(response.balances[0].denom, "tcy");
    assert_eq!(response.balances[0].balance, 500);
}

#[test]
fn test_has_exceeded_debt_limit() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["tcy".to_string()];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();

    let user_address = api.addr_make("user");

    // Test user with no debt (should return false)
    let query_msg = auto_fee_manager::msg::QueryMsg::HasExceededDebtLimit {
        user: user_address.clone(),
    };
    let result = auto_fee_manager::contract::query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let has_exceeded: bool = cosmwasm_std::from_json(result).unwrap();
    assert_eq!(has_exceeded, false);

    // Create debt for the user by charging fees without deposit
    let creator_address = api.addr_make("creator");
    let test_fees = create_test_user_fees_with_creator(
        user_address.clone(),
        creator_address,
        "tcy".to_string(),
        Uint128::from(1500u128), // More than max_debt
    );

    execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        vec![test_fees],
    ).unwrap();

    // Test user with debt exceeding limit (should return true)
    let query_msg = auto_fee_manager::msg::QueryMsg::HasExceededDebtLimit {
        user: user_address.clone(),
    };
    let result = auto_fee_manager::contract::query(deps.as_ref(), env, query_msg).unwrap();
    let has_exceeded: bool = cosmwasm_std::from_json(result).unwrap();
    assert_eq!(has_exceeded, true);
}

#[test]
fn test_get_non_creator_fees() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["tcy".to_string()];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();

    // Query non-creator fees when there are no fees
    let query_msg = auto_fee_manager::msg::QueryMsg::GetNonCreatorFees {};
    let result = auto_fee_manager::contract::query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let response: auto_fee_manager::msg::NonCreatorFeesResponse = cosmwasm_std::from_json(result).unwrap();

    // Verify response - should have no fees initially
    assert_eq!(response.execution_fees.len(), 0);
    assert_eq!(response.distribution_fees.len(), 0);

    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "tcy".to_string(),
        amount: Uint128::from(1000u128),
    }];
    let deposit_info = cosmwasm_std::testing::message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();

    // Create test fees with Execution type to generate execution fees
    let test_fees = UserFees {
        user: user_address.clone(),
        fees: vec![
            Fee {
                timestamp: 1234567890,
                amount: Uint128::from(100u128),
                denom: "tcy".to_string(),
                fee_type: FeeType::Execution,
                creator_address: None,
            },
        ],
    };

    // Charge fees from user balance to generate execution fees
    execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        vec![test_fees],
    ).unwrap();

    // Query non-creator fees again
    let query_msg = auto_fee_manager::msg::QueryMsg::GetNonCreatorFees {};
    let result = auto_fee_manager::contract::query(deps.as_ref(), env, query_msg).unwrap();
    let response: auto_fee_manager::msg::NonCreatorFeesResponse = cosmwasm_std::from_json(result).unwrap();

    // Verify response - should have execution fees now
    assert_eq!(response.execution_fees.len(), 1);
    assert_eq!(response.execution_fees[0].denom, "tcy");
    assert!(response.execution_fees[0].balance > Uint128::zero());
    assert_eq!(response.distribution_fees.len(), 0); // No distribution fees yet
} 