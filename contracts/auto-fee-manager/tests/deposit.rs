use cosmwasm_std::{testing::{mock_dependencies, mock_env, message_info}};
use auto_fee_manager::{msg::AcceptedDenom, ContractError};
use cosmwasm_std::{Coin, Uint128};
mod utils;
use crate::utils::*;

#[test]
fn test_deposit_with_valid_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test deposit
    let user_address = api.addr_make("user");
    let funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    }];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify response
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/deposit");
    assert_eq!(response.events[0].attributes.len(), 2);
    assert_eq!(response.events[0].attributes[0].key, "user");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "funds");
    assert!(response.events[0].attributes[1].value.contains("uusdc"));
}
#[test]
fn test_deposit_without_funds_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test deposit without funds
    let user_address = api.addr_make("user");
    let info = message_info(&user_address, &[]);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    );
    // Verify error
    assert!(result.is_err());
    match result {
        Err(ContractError::NoFundsSent {}) => {},
        _ => panic!("Expected NoFundsSent error"),
    }
}
#[test]
fn test_deposit_with_invalid_denom_fails() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test deposit with invalid denom
    let user_address = api.addr_make("user");
    let funds = vec![Coin {
        denom: "uatom".to_string(), // Not accepted
        amount: Uint128::from(100u128),
    }];
    let info = message_info(&user_address, &funds);
    let result = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
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
fn test_deposit_multiple_denoms() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![
        AcceptedDenom {
            denom: "uusdc".to_string(),
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        },  
        AcceptedDenom {
            denom: "uatom".to_string(),
            max_debt: Uint128::zero(),
            min_balance_threshold: Uint128::zero(),
        },
    ];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    // Test deposit with multiple denoms
    let user_address = api.addr_make("user");
    let funds = vec![
        Coin {
            denom: "uusdc".to_string(),
            amount: Uint128::from(100u128),
        },
        Coin {
            denom: "uatom".to_string(),
            amount: Uint128::from(50u128),
        }
    ];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify response
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/deposit");
    assert_eq!(response.events[0].attributes.len(), 2);
    assert_eq!(response.events[0].attributes[0].key, "user");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "funds");
    assert!(response.events[0].attributes[1].value.contains("uusdc"));
    assert!(response.events[0].attributes[1].value.contains("uatom"));
}
#[test]
fn test_deposit_balance_tracking() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![AcceptedDenom {
        denom: "uusdc".to_string(),
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    let user_address = api.addr_make("user");
    // First deposit
    let funds1 = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(100u128),
    }];
    let info1 = message_info(&user_address, &funds1);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env.clone(),
        info1,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Second deposit
    let funds2 = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(50u128),
    }];
    let info2 = message_info(&user_address, &funds2);
    auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info2,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify balance is cumulative (150 uusdc)
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 150);
}
#[test]
fn test_deposit_event_balance_turned_positive() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![
        AcceptedDenom {
            denom: "uusdc".to_string(),
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        },  
        AcceptedDenom {
            denom: "uatom".to_string(),
            max_debt: Uint128::zero(),
            min_balance_threshold: Uint128::zero(),
        },
    ];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    let user_address = api.addr_make("user");
    // First, create negative balance by charging fees (simulating debt)
    // This would normally be done through the charge fees function, but for testing
    // we'll directly set a negative balance
    auto_fee_manager::state::USER_BALANCES
        .save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-100)
        .unwrap();
    // Now deposit enough to turn balance positive
    let funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(150u128), // This will turn -100 to +50
    }];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify event was emitted
    assert_eq!(response.events.len(), 2);
    let event = &response.events[1];
    assert_eq!(event.ty, "autorujira-fee-manager/deposit_completed");
    let user_attr = event.attributes.iter().find(|attr| attr.key == "user").unwrap();
    assert_eq!(user_attr.value, user_address.to_string());
    let balances_attr = event.attributes.iter().find(|attr| attr.key == "balances_turned_positive").unwrap();
    assert_eq!(balances_attr.value, "uusdc");
    // Verify final balance is positive
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 50);
}
#[test]
fn test_deposit_event_multiple_balances_turned_positive() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![
        AcceptedDenom {
            denom: "uusdc".to_string(),
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        },  
        AcceptedDenom {
            denom: "uatom".to_string(),
            max_debt: Uint128::zero(),
            min_balance_threshold: Uint128::zero(),
        },
    ];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    let user_address = api.addr_make("user");
    // Set negative balances for both denoms
    auto_fee_manager::state::USER_BALANCES
        .save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-100)
        .unwrap();
    auto_fee_manager::state::USER_BALANCES
        .save(deps.as_mut().storage, (user_address.clone(), "uatom"), &-50)
        .unwrap();
    // Deposit enough to turn both balances positive
    let funds = vec![
        Coin {
            denom: "uusdc".to_string(),
            amount: Uint128::from(150u128), // -100 + 150 = +50
        },
        Coin {
            denom: "uatom".to_string(),
            amount: Uint128::from(100u128), // -50 + 100 = +50
        }
    ];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify event was emitted with both denoms
    assert_eq!(response.events.len(), 2);
    let event = &response.events[1];
    assert_eq!(event.ty, "autorujira-fee-manager/deposit_completed");
    let balances_attr = event.attributes.iter().find(|attr| attr.key == "balances_turned_positive").unwrap();
    // Order might vary, so check both possibilities
    assert!(balances_attr.value == "uusdc,uatom" || balances_attr.value == "uatom,uusdc");
    // Verify final balances are positive
    let balance_uusdc = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address.clone(), "uusdc"))
        .unwrap();
    let balance_uatom = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uatom"))
        .unwrap();
    assert_eq!(balance_uusdc, 50);
    assert_eq!(balance_uatom, 50);
}
#[test]
fn test_deposit_no_event_when_balance_stays_negative() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![
        AcceptedDenom {
            denom: "uusdc".to_string(),
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        },  
    ];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    let user_address = api.addr_make("user");
    // Set negative balance
    auto_fee_manager::state::USER_BALANCES
        .save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &-100)
        .unwrap();
    // Deposit but not enough to turn positive
    let funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(50u128), // -100 + 50 = -50 (still negative)
    }];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify no event was emitted
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/deposit");
    assert_eq!(response.events[0].attributes.len(), 2);
    assert_eq!(response.events[0].attributes[0].key, "user");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "funds");
    assert!(response.events[0].attributes[1].value.contains("uusdc"));

    // Verify balance is still negative
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, -50);
}
#[test]
fn test_deposit_no_event_when_balance_was_already_positive() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_destination");
    let crank_authorized_address = api.addr_make("crank_authorized");
    let workflow_manager_address = api.addr_make("workflow_manager");
    let accepted_denoms = vec![
        AcceptedDenom {
            denom: "uusdc".to_string(),
            max_debt: Uint128::from(1000u128),
            min_balance_threshold: Uint128::from(100u128),
        },      ];
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        accepted_denoms,
        execution_fees_destination_address.clone(),
        execution_fees_destination_address, // Using same address for distribution fees for testing,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5% distribution fee
    ).unwrap();
    let user_address = api.addr_make("user");
    // Set positive balance
    auto_fee_manager::state::USER_BALANCES
        .save(deps.as_mut().storage, (user_address.clone(), "uusdc"), &100)
        .unwrap();
    // Deposit more (balance stays positive)
    let funds = vec![Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::from(50u128), // 100 + 50 = 150 (still positive)
    }];
    let info = message_info(&user_address, &funds);
    let response = auto_fee_manager::contract::execute(
        deps.as_mut(),
        env,
        info,
        auto_fee_manager::msg::ExecuteMsg::Deposit {},
    ).unwrap();
    // Verify no event was emitted
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/deposit");
    assert_eq!(response.events[0].attributes.len(), 2);
    assert_eq!(response.events[0].attributes[0].key, "user");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "funds");
    assert!(response.events[0].attributes[1].value.contains("uusdc"));

    // Verify balance increased
    let balance = auto_fee_manager::state::USER_BALANCES
        .load(deps.as_ref().storage, (user_address, "uusdc"))
        .unwrap();
    assert_eq!(balance, 150);
} 