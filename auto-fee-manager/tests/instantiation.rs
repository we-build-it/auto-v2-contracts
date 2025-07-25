use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use auto_fee_manager::ContractError;
use cosmwasm_std::{Addr, Coin, Uint128};

mod utils;
use utils::{
    instantiate_contract
};

#[test]
fn test_instantiate_with_valid_parameters() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let gas_destination_address = api.addr_make("gas_destination");
    let infra_destination_address = api.addr_make("infra_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["uusdc".to_string(), "uatom".to_string()];
    
    // Initialize contract using utils function
    let response = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        max_debt.clone(),
        min_balance_threshold.clone(),
        gas_destination_address.clone(),
        infra_destination_address.clone(),
        accepted_denoms.clone(),
        crank_authorized_address.clone(),
        workflow_manager_address.clone(),
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
    let gas_destination_address = api.addr_make("gas_destination");
    let infra_destination_address = api.addr_make("infra_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::zero(), // Zero should be allowed
    };
    let min_balance_threshold = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address,
        workflow_manager_address,
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
    let gas_destination_address = api.addr_make("gas_destination");
    let infra_destination_address = api.addr_make("infra_destination");
    let crank_authorized_address = Addr::unchecked(""); // Empty address
    let workflow_manager_address = api.addr_make("workflow_manager");
    
    let max_debt = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        max_debt.clone(),
        min_balance_threshold.clone(),
        gas_destination_address.clone(),
        infra_destination_address.clone(),
        accepted_denoms.clone(),
        crank_authorized_address,
        workflow_manager_address.clone(),
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
    let gas_destination_address = api.addr_make("gas_destination");
    let infra_destination_address = api.addr_make("infra_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = Addr::unchecked(""); // Empty address
    
    let max_debt = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(1000u128),
    };
    let min_balance_threshold = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    };
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    let result = instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        max_debt.clone(),
        min_balance_threshold.clone(),
        gas_destination_address.clone(),
        infra_destination_address.clone(),
        accepted_denoms.clone(),
        crank_authorized_address,
        workflow_manager_address,
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