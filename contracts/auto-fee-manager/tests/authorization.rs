use std::collections::HashMap;

use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use auto_fee_manager::{msg::AcceptedDenomValue, ContractError};
use cosmwasm_std::{Uint128};
mod utils;
use crate::utils::*;

// TODO: This test is testing the different authorizations wrongly, we need to fix this as follows:
// - Workflow Manager: handle_charge_fees_from_user_balance

#[test]
fn test_crank_authorized_functions_require_authorization() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address.clone(),
        workflow_manager_address.clone(),
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test that unauthorized address cannot call crank functions
    let unauthorized_address = api.addr_make("unauthorized");
    // Test DistributeCreatorFees
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env.clone(),
        unauthorized_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, unauthorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }
    // Test DistributeNonCreatorFees
    let result = execute_distribute_non_creator_fees(
        deps.as_mut(),
        env.clone(),
        unauthorized_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, unauthorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }
    // Test that authorized address can call crank functions
    // Test DistributeCreatorFees - should fail when no creator fees exist
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NoCreatorFeesToDistribute {}) => {
            // Expected error when no creator fees exist
        },
        _ => panic!("Expected NoCreatorFeesToDistribute error"),
    }
    // Test DistributeNonCreatorFees - should fail when no execution fees exist
    let result = execute_distribute_non_creator_fees(
        deps.as_mut(),
        env,
        crank_authorized_address,
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NoExecutionFeesToDistribute {}) => {
            // Expected error when no execution fees exist
        },
        _ => panic!("Expected NoExecutionFeesToDistribute error"),
    }
}

#[test]
fn test_workflow_manager_functions_require_authorization() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address.clone(),
        workflow_manager_address.clone(),
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test that unauthorized address cannot call workflow manager functions

    let unauthorized_address = api.addr_make("unauthorized");
    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    // Test ChargeFeesFromUserBalance
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        unauthorized_address.clone(),
        vec![test_user_fees.clone()],
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, unauthorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }

    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    // Test ChargeFeesFromUserBalance
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        workflow_manager_address.clone(),
        vec![test_user_fees],
    );
    assert!(result.is_ok());
}

#[test]
fn test_sudo_set_crank_authorized_address() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Change crank authorized address via sudo
    let new_crank_authorized_address = api.addr_make("new_crank_authorized");
    let result = sudo_set_crank_authorized_address(
        deps.as_mut(),
        env.clone(),
        new_crank_authorized_address.clone(),
    );
    assert!(result.is_ok());

    // Test that old crank authorized address can no longer call restricted functions
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, crank_authorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }
    // Test that new crank authorized address can call restricted functions
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env.clone(),
        new_crank_authorized_address.clone(),
    );

    assert!(result.is_err());
    match result {
        Err(ContractError::NoCreatorFeesToDistribute {}) => {
            // Expected error when no creator fees exist
        },
        _ => panic!("Expected NoCreatorFeesToDistribute error"),
    }
}

#[test]
fn test_sudo_set_workflow_manager_address() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address.clone(),
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Change workflow manager address via sudo
    let new_workflow_manager_address = api.addr_make("new_workflow_manager");
    let result = sudo_set_workflow_manager_address(
        deps.as_mut(),
        env.clone(),
        new_workflow_manager_address.clone(),
    );
    assert!(result.is_ok());
    // Test that old workflow manager address can no longer call restricted functions
    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        workflow_manager_address.clone(),
        vec![test_user_fees.clone()],
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, workflow_manager_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }
    // Test that new workflow manager address can call restricted functions
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        new_workflow_manager_address,
        vec![test_user_fees],
    );
    assert!(result.is_ok());
}

#[test]
fn test_sudo_set_execution_fees_destination_address() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Change execution fees destination address via sudo
    let new_execution_fees_destination_address = api.addr_make("new_execution_destination");
    let result = sudo_set_execution_fees_destination_address(
        deps.as_mut(),
        env.clone(),
        new_execution_fees_destination_address.clone(),
    );
    assert!(result.is_ok());
    // Verify response event and attributes
    let response = result.unwrap();
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/sudo_set_execution_fees_destination_address");
    assert_eq!(response.events[0].attributes.len(), 0);
}

#[test]
fn test_sudo_set_distribution_fees_destination_address() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address,
        distribution_fees_destination_address.clone(),
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Change distribution fees destination address via sudo
    let new_distribution_fees_destination_address = api.addr_make("new_distribution_destination");
    let result = sudo_set_distribution_fees_destination_address(
        deps.as_mut(),
        env.clone(),
        new_distribution_fees_destination_address.clone(),
    );
    assert!(result.is_ok());
    // Verify response attributes
    let response = result.unwrap();
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/sudo_set_distribution_fees_destination_address");
    assert_eq!(response.events[0].attributes.len(), 0);
}

#[test]
fn test_has_exceeded_debt_limit() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract with max_debt of 1000 uusdc
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![(
        "uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }
    )].into_iter().collect();
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    
    let user_address = api.addr_make("user");
    
    // Test 1: User with positive balance (no debt)
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &500).unwrap();
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, false);
    
    // Test 2: User with zero balance (no debt)
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &0).unwrap();
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, false);
    
    // Test 3: User with small debt (within limit)
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-500).unwrap();
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, false);
    
    // Test 4: User with debt at the limit
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-1000).unwrap();
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, false);
    
    // Test 5: User with debt exceeding the limit
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-1500).unwrap();
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, true);
    
    // Test 6: User with no balance record (should default to 0)
    auto_fee_manager::state::USER_BALANCES.remove(deps.as_mut().storage, (user_address.clone(), "uusdc"));
    let result = auto_fee_manager::handlers::has_exceeded_debt_limit(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result, false);
}

#[test]
fn test_get_user_balances() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract with multiple accepted denoms
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![
        ("uusdc".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        }),
        ("uatom".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::zero(),
            min_balance_threshold: Uint128::zero(),
        }),
        ("uosmo".to_string(),
        AcceptedDenomValue {
            max_debt: Uint128::zero(),
            min_balance_threshold: Uint128::zero(),
        }),
    ].into_iter().collect();    

    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    
    let user_address = api.addr_make("user");
    
    // Test 1: User with no balances (should return all denoms with 0 balance)
    let result = auto_fee_manager::handlers::get_user_balances(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result.user, user_address);
    assert_eq!(result.balances.len(), 3);
    
    // Check that all accepted denoms are present with 0 balance
    let denoms: Vec<String> = result.balances.iter().map(|b| b.denom.clone()).collect();
    assert!(denoms.contains(&"uusdc".to_string()));
    assert!(denoms.contains(&"uatom".to_string()));
    assert!(denoms.contains(&"uosmo".to_string()));
    
    for balance in &result.balances {
        assert_eq!(balance.balance, 0);
    }
    
    // Test 2: User with some balances set
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &500).unwrap();
    auto_fee_manager::state::USER_BALANCES.save(deps.as_mut().storage, (user_address.clone(), "uatom"), &-200).unwrap();
    // uosmo remains at 0 (no record)
    
    let result = auto_fee_manager::handlers::get_user_balances(deps.as_ref(), user_address.clone()).unwrap();
    assert_eq!(result.user, user_address);
    assert_eq!(result.balances.len(), 3);
    
    // Find and verify each balance
    let mut uusdc_balance = None;
    let mut uatom_balance = None;
    let mut uosmo_balance = None;
    
    for balance in &result.balances {
        match balance.denom.as_str() {
            "uusdc" => uusdc_balance = Some(balance.balance),
            "uatom" => uatom_balance = Some(balance.balance),
            "uosmo" => uosmo_balance = Some(balance.balance),
            _ => panic!("Unexpected denom: {}", balance.denom),
        }
    }
    
    assert_eq!(uusdc_balance, Some(500));
    assert_eq!(uatom_balance, Some(-200));
    assert_eq!(uosmo_balance, Some(0));
} 