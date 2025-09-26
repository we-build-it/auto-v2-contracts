use std::collections::HashMap;

use cosmwasm_std::{testing::{mock_dependencies, mock_env, message_info}};
use auto_fee_manager::{msg::AcceptedDenomValue, ContractError};
use cosmwasm_std::{Coin, Uint128};
mod utils;
use utils::{
    instantiate_contract
};
#[test]
fn test_withdraw_with_sufficient_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
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
    let user_address = api.addr_make("user");
    // First, deposit some funds to create a balance
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(200u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Now withdraw some funds
    let withdraw_amount = Uint128::from(100u128);
    let withdraw_info = message_info(&user_address, &[]);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        withdraw_info,
        auto_fee_manager::msg::ExecuteMsg::Withdraw {
            denom: "uusdc".to_string(),
            amount: withdraw_amount,
        },
    ).unwrap();
    // Verify response attributes
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/withdraw");
    assert_eq!(response.events[0].attributes.len(), 4);
    assert_eq!(response.events[0].attributes[0].key, "user");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "denom");
    assert_eq!(response.events[0].attributes[1].value, "uusdc");
    assert_eq!(response.events[0].attributes[2].key, "amount");
    assert_eq!(response.events[0].attributes[2].value, "100");
    assert_eq!(response.events[0].attributes[3].key, "new_balance");
    assert_eq!(response.events[0].attributes[3].value, "100");
    // Verify bank message was added
    assert_eq!(response.messages.len(), 1);
    // Note: We don't verify the exact bank message content as it's complex to test
    // The important thing is that a message was added for the bank transfer
    // Verify balance was updated
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 100);
}
#[test]
fn test_withdraw_with_insufficient_balance_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
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
    let user_address = api.addr_make("user");
    // Try to withdraw without having any balance
    let withdraw_amount = Uint128::from(100u128);
    let withdraw_info = message_info(&user_address, &[]);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        withdraw_info,
        auto_fee_manager::msg::ExecuteMsg::Withdraw {
            denom: "uusdc".to_string(),
            amount: withdraw_amount,
        },
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::InsufficientBalance { available, requested }) => {
            assert_eq!(available, 0);
            assert_eq!(requested, withdraw_amount);
        },
        _ => panic!("Expected InsufficientBalance error"),
    }
}
#[test]
fn test_withdraw_with_invalid_denom_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
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
    let user_address = api.addr_make("user");
    // Try to withdraw invalid denom
    let withdraw_amount = Uint128::from(100u128);
    let withdraw_info = message_info(&user_address, &[]);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        withdraw_info,
        auto_fee_manager::msg::ExecuteMsg::Withdraw {
            denom: "uatom".to_string(), // Not accepted
            amount: withdraw_amount,
        },
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
fn test_withdraw_zero_amount_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
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
    let user_address = api.addr_make("user");
    // Try to withdraw zero amount
    let withdraw_info = message_info(&user_address, &[]);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        withdraw_info,
        auto_fee_manager::msg::ExecuteMsg::Withdraw {
            denom: "uusdc".to_string(),
            amount: Uint128::zero(),
        },
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidWithdrawalAmount {}) => {},
        _ => panic!("Expected InvalidWithdrawalAmount error"),
    }
}
#[test]
fn test_withdraw_negative_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
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
    let user_address = api.addr_make("user");
    // First, deposit some funds
    let deposit_funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    }];
    let deposit_info = message_info(&user_address, &deposit_funds);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        deposit_info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Withdraw more than available (should fail)
    let withdraw_amount = Uint128::from(200u128);
    let withdraw_info = message_info(&user_address, &[]);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        withdraw_info,
        auto_fee_manager::msg::ExecuteMsg::Withdraw {
            denom: "uusdc".to_string(),
            amount: withdraw_amount,
        },
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::InsufficientBalance { available, requested }) => {
            assert_eq!(available, 100);
            assert_eq!(requested, withdraw_amount);
        },
        _ => panic!("Expected InsufficientBalance error"),
    }
    // Verify balance was not changed
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 100);
} 