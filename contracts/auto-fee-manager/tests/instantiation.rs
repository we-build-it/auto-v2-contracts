use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use auto_fee_manager::{msg::AcceptedDenom, ContractError};
use cosmwasm_std::{Addr, Uint128};

mod utils;
use crate::utils::*;


#[test]
fn test_instantiate_with_valid_parameters() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    let response = instantiate_contract(
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

    // Verify response attributes
    assert_eq!(response.attributes.len(), 0);
}

#[test]
fn test_instantiate_with_zero_max_debt_succeeds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::zero(),
        min_balance_threshold: Uint128::from(100u128),
    }];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    );
    
    // Verify that the operation succeeds
    assert!(result.is_ok());
}

#[test]
fn test_instantiate_with_empty_crank_authorized_address_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = Addr::unchecked(""); // Empty address
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address.clone(),
        Uint128::from(5u128), // 5% distribution fee
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    // Check that the error is the expected one
    match result {
        Err(ContractError::InvalidAddress { reason }) => {
            assert_eq!(reason, "authorized_address cannot be empty");
        }
        _ => panic!("Expected InvalidAddress error, got different error"),
    }
}

#[test]
fn test_instantiate_with_empty_workflow_manager_address_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = Addr::unchecked(""); // Empty address
    
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        accepted_denoms,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    // Check that the error is the expected one
    match result {
        Err(ContractError::InvalidAddress { reason }) => {
            assert_eq!(reason, "workflow_manager_address cannot be empty");
        }
        _ => panic!("Expected InvalidAddress error, got different error"),
    }
} 