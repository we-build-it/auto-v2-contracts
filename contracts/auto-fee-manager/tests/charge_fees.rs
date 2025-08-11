use cosmwasm_std::{testing::{mock_dependencies, mock_env, message_info}};
use auto_fee_manager::ContractError;
use cosmwasm_std::{Coin, Uint128};
use cosmwasm_std::{CosmosMsg, BankMsg};
mod utils;
use crate::utils::*;

#[test]
fn test_charge_fees_from_user_balance_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(500u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Create test fees for the user
    let test_user_fees = create_test_user_fees(user_address.clone());
    // Charge fees from user balance
    let response = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    ).unwrap();
    // Verify response attributes
    assert_eq!(response.attributes.len(), 2);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "charge_fees_from_user_balance");
    assert_eq!(response.attributes[1].key, "batch_size");
    assert_eq!(response.attributes[1].value, "1");
    // Verify user balance was reduced
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, -2500); // 500 - 1000 - 2000 = -2500
}
#[test]
fn test_charge_fees_from_user_balance_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Try to charge fees with unauthorized address
    let unauthorized_address = api.addr_make("unauthorized");
    let user_address = api.addr_make("user");
    let test_user_fees = create_test_user_fees(user_address);
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        unauthorized_address.clone(),
        vec![test_user_fees],
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { address }) => {
            assert_eq!(address, unauthorized_address.to_string());
        },
        _ => panic!("Expected NotAuthorized error"),
    }
}
#[test]
fn test_charge_fees_from_user_balance_invalid_denom() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let accepted_denoms = vec!["uusdc".to_string()]; // Only uusdc accepted
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create test fees with invalid denom
    let user_address = api.addr_make("user");
    let test_user_fees = auto_fee_manager::msg::UserFees {
        user: user_address,
        fees: vec![
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test execution fee".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(1000u128),
                denom: "uatom".to_string(), // Invalid denom
                fee_type: auto_fee_manager::msg::FeeType::Execution,
                creator_address: None,
            }
        ],
    };
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::DenomNotAccepted { denom }) => {
            assert_eq!(denom, "uatom");
        },
        _ => panic!("Expected DenomNotAccepted error"),
    }
}
#[test]
fn test_charge_fees_from_user_balance_below_threshold_event() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(150u128), // Just above threshold
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Create test fees that will bring balance below threshold
    let test_user_fees = auto_fee_manager::msg::UserFees {
        user: user_address.clone(),
        fees: vec![
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test execution fee".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(100u128), // This will bring balance to 50, below threshold
                denom: "uusdc".to_string(),
                fee_type: auto_fee_manager::msg::FeeType::Execution,
                creator_address: None,
            }
        ],
    };
    // Charge fees from user balance
    let response = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    ).unwrap();
    // Verify event was emitted
    assert_eq!(response.events.len(), 1);
    let event = &response.events[0];
    assert_eq!(event.ty, "balance_below_threshold");
    assert_eq!(event.attributes.len(), 2);
    assert_eq!(event.attributes[0].key, "user");
    assert_eq!(event.attributes[0].value, user_address.to_string());
    assert_eq!(event.attributes[1].key, "denom");
    assert_eq!(event.attributes[1].value, "uusdc");
    // Verify user balance was reduced
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 50); // 150 - 100 = 50
} 
#[test]
fn test_charge_fees_from_user_balance_storage_tracking() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(1000u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Create test fees with both Execution and Creator types
    let creator_address = api.addr_make("creator");
    let test_user_fees = auto_fee_manager::msg::UserFees {
        user: user_address.clone(),
        fees: vec![
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test execution fee".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(100u128),
                denom: "uusdc".to_string(),
                fee_type: auto_fee_manager::msg::FeeType::Execution,
                creator_address: None,
            },
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-2".to_string(),
                action_id: "test-action-2".to_string(),
                description: "Test creator fee".to_string(),
                timestamp: 1234567891,
                amount: Uint128::from(200u128),
                denom: "uusdc".to_string(),
                fee_type: auto_fee_manager::msg::FeeType::Creator,
                creator_address: Some(creator_address.clone()),
            },
        ],
    };
    // Charge fees from user balance
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    );
    assert!(result.is_ok());
    // Verify user balance was reduced
    let user_balance = auto_fee_manager::state::USER_BALANCES
        .may_load(deps.as_ref().storage, (user_address.clone(), "uusdc"))
        .unwrap()
        .unwrap_or(0);
    assert_eq!(user_balance, 700); // 1000 - 100 - 200
    // Verify execution fees were added to storage
    let execution_fees = auto_fee_manager::state::EXECUTION_FEES
        .may_load(deps.as_ref().storage, "uusdc")
        .unwrap()
        .unwrap_or(Uint128::zero());
    assert_eq!(execution_fees, Uint128::from(100u128));
    // Verify creator fees were added to storage
    let creator_fees = auto_fee_manager::state::CREATOR_FEES
        .may_load(deps.as_ref().storage, (&creator_address, "uusdc"))
        .unwrap()
        .unwrap_or(Uint128::zero());
    assert_eq!(creator_fees, Uint128::from(200u128));
} 
#[test]
fn test_charge_fees_from_user_balance_creator_fallback() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create a user and give them some balance
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(1000u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Create test fees with Creator type but no creator_address (should fail)
    let test_user_fees = auto_fee_manager::msg::UserFees {
        user: user_address.clone(),
        fees: vec![
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test creator fee with fallback".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(300u128),
                denom: "uusdc".to_string(),
                fee_type: auto_fee_manager::msg::FeeType::Creator,
                creator_address: None, // Should fail - creator_address is required
            },
        ],
    };
    // Charge fees from user balance - should fail
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidCreatorAddress { reason }) => {
            assert_eq!(reason, "creator_address is required for Creator fees");
        },
        _ => panic!("Expected InvalidCreatorAddress error"),
    }
} 
#[test]
fn test_charge_fees_from_user_balance_partial_execution_fee() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Create a user and give them some balance (10 uusdc)
    let user_address = api.addr_make("user");
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(10u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Try to charge a fee of 15 uusdc (more than the user has)
    let test_user_fees = auto_fee_manager::msg::UserFees {
        user: user_address.clone(),
        fees: vec![
            auto_fee_manager::msg::Fee {
                workflow_instance_id: "test-instance-1".to_string(),
                action_id: "test-action-1".to_string(),
                description: "Test partial execution fee".to_string(),
                timestamp: 1234567890,
                amount: Uint128::from(15u128),
                denom: "uusdc".to_string(),
                fee_type: auto_fee_manager::msg::FeeType::Execution,
                creator_address: None,
            },
        ],
    };
    let result = execute_charge_fees_from_user_balance(
        deps.as_mut(),
        env,
        crank_authorized_address,
        vec![test_user_fees],
    );
    assert!(result.is_ok());
    // The user's balance should now be -5
    let user_balance = auto_fee_manager::state::USER_BALANCES
        .may_load(deps.as_ref().storage, (user_address.clone(), "uusdc"))
        .unwrap()
        .unwrap_or(0);
    assert_eq!(user_balance, -5);
    // Only 10 uusdc should have been added to execution fees
    let execution_fees = auto_fee_manager::state::EXECUTION_FEES
        .may_load(deps.as_ref().storage, "uusdc")
        .unwrap()
        .unwrap_or(Uint128::zero());
    assert_eq!(execution_fees, Uint128::from(10u128));
} 
#[test]
fn test_distribute_non_creator_fees() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
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
    let distribution_fees_destination_address = api.addr_make("distribution_destination");
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        max_debt,
        min_balance_threshold,
        execution_fees_destination_address.clone(),
        distribution_fees_destination_address,
        accepted_denoms,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Add some execution fees to the contract
    auto_fee_manager::state::EXECUTION_FEES.save(deps.as_mut().storage, "uusdc", &Uint128::from(500u128)).unwrap();
    auto_fee_manager::state::EXECUTION_FEES.save(deps.as_mut().storage, "uatom", &Uint128::from(300u128)).unwrap();
    // Test that unauthorized address cannot distribute
    let unauthorized_address = api.addr_make("unauthorized");
    let result = execute_distribute_non_creator_fees(
        deps.as_mut(),
        env.clone(),
        unauthorized_address,
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NotAuthorized { .. }) => {
            // Expected error
        },
        _ => panic!("Expected NotAuthorized error"),
    }
    // Test successful distribution by authorized address
    let result = execute_distribute_non_creator_fees(
        deps.as_mut(),
        env,
        crank_authorized_address,
    );
    assert!(result.is_ok());
    // Verify that execution fees were cleared
    let uusdc_fees = auto_fee_manager::state::EXECUTION_FEES.may_load(deps.as_ref().storage, "uusdc").unwrap();
    let uatom_fees = auto_fee_manager::state::EXECUTION_FEES.may_load(deps.as_ref().storage, "uatom").unwrap();
    assert_eq!(uusdc_fees, None);
    assert_eq!(uatom_fees, None);
    // Verify response attributes
    let response = result.unwrap();
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "distribute_non_creator_fees");
    assert_eq!(response.attributes[1].key, "execution_destination");
    assert_eq!(response.attributes[1].value, execution_fees_destination_address.to_string());
    // Verify bank messages were created
    assert_eq!(response.messages.len(), 2);
    // Check messages (order is alphabetical by denom: uatom, then uusdc)
    // First message (uatom)
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &response.messages[0].msg {
        assert_eq!(to_address, &execution_fees_destination_address.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, "uatom");
        assert_eq!(amount[0].amount, Uint128::from(300u128));
    } else {
        panic!("Expected BankMsg::Send");
    }
    // Second message (uusdc)
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &response.messages[1].msg {
        assert_eq!(to_address, &execution_fees_destination_address.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, "uusdc");
        assert_eq!(amount[0].amount, Uint128::from(500u128));
    } else {
        panic!("Expected BankMsg::Send");
    }
} 