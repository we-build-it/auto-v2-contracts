use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use auto_fee_manager::ContractError;
use cosmwasm_std::{Coin, Uint128};

mod utils;
use utils::{
    instantiate_contract, execute_charge_fees_from_user_balance, 
    execute_charge_fees_from_message_coins, execute_distribute_creator_fees,
    execute_distribute_non_creator_fees, sudo_set_crank_authorized_address,
    sudo_set_workflow_manager_address, create_test_user_fees
};

#[test]
fn test_crank_authorized_functions_require_authorization() {
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
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address.clone(),
    ).unwrap();

    // Test that unauthorized address cannot call crank functions
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
    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    
    // Test ChargeFeesFromUserBalance
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        vec![test_user_fees],
    );
    assert!(result.is_ok());

    // Test DistributeCreatorFees
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
    );
    assert!(result.is_ok());

    // Test DistributeNonCreatorFees
    let result = execute_distribute_non_creator_fees(
        deps.as_mut(),
        env,
        crank_authorized_address,
    );
    assert!(result.is_ok());
}

#[test]
fn test_workflow_manager_functions_require_authorization() {
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
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address.clone(),
    ).unwrap();

    // Test that unauthorized address cannot call workflow manager functions
    let unauthorized_address = api.addr_make("unauthorized");
    let test_fees = vec![
        auto_fee_manager::msg::Fee {
            workflow_instance_id: "test-instance-1".to_string(),
            action_id: "test-action-1".to_string(),
            description: "Test execution fee".to_string(),
            timestamp: 1234567890,
            amount: Uint128::from(1000u128),
            denom: "uusdc".to_string(),
            fee_type: auto_fee_manager::msg::FeeType::Execution,
            creator_address: None,
        }
    ];
    let creator_address = api.addr_make("creator");
    
    // Test ChargeFeesFromMessageCoins
    let result = execute_charge_fees_from_message_coins(
        deps.as_mut(),
        env.clone(),
        unauthorized_address.clone(),
        test_fees.clone(),
        creator_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, unauthorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }

    // Test that crank authorized address cannot call workflow manager functions
    let result = execute_charge_fees_from_message_coins(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        test_fees.clone(),
        creator_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, crank_authorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }

    // Test that workflow manager can call ChargeFeesFromMessageCoins
    let result = execute_charge_fees_from_message_coins(
        deps.as_mut(),
        env,
        workflow_manager_address.clone(),
        test_fees,
        creator_address,
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
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
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
    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env.clone(),
        crank_authorized_address.clone(),
        vec![test_user_fees],
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, crank_authorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }

    // Test that new crank authorized address can call restricted functions
    let test_user_fees = create_test_user_fees(api.addr_make("user"));
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        new_crank_authorized_address,
        vec![test_user_fees],
    );
    assert!(result.is_ok());
}

#[test]
fn test_sudo_set_workflow_manager_address() {
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
    let accepted_denoms = vec!["uusdc".to_string()];
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        gas_destination_address,
        infra_destination_address,
        accepted_denoms,
        crank_authorized_address,
        workflow_manager_address.clone(),
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
    let test_fees = vec![
        auto_fee_manager::msg::Fee {
            workflow_instance_id: "test-instance-1".to_string(),
            action_id: "test-action-1".to_string(),
            description: "Test execution fee".to_string(),
            timestamp: 1234567890,
            amount: Uint128::from(1000u128),
            denom: "uusdc".to_string(),
            fee_type: auto_fee_manager::msg::FeeType::Execution,
            creator_address: None,
        }
    ];
    let creator_address = api.addr_make("creator");
    
    let result = execute_charge_fees_from_message_coins(
        deps.as_mut(),
        env.clone(),
        workflow_manager_address.clone(),
        test_fees.clone(),
        creator_address.clone(),
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, workflow_manager_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }

    // Test that new workflow manager address can call restricted functions
    let result = execute_charge_fees_from_message_coins(
        deps.as_mut(),
        env,
        new_workflow_manager_address,
        test_fees,
        creator_address,
    );
    assert!(result.is_ok());
} 