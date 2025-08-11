use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{Coin, Uint128};
use auto_fee_manager::ContractError;
use cosmwasm_std::{CosmosMsg, BankMsg};
mod utils;
use crate::utils::*;

#[test]
fn test_claim_creator_fees_no_fees() {
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
    // Test claiming when no fees exist
    let creator_address = api.addr_make("creator");
    let result = execute_claim_creator_fees(
        deps.as_mut(),
        env,
        creator_address,
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NoCreatorFeesToClaim {}) => {
            // Expected error
        },
        _ => panic!("Expected NoCreatorFeesToClaim error"),
    }
}
#[test]
fn test_claim_creator_fees_success() {
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
    // Add some creator fees
    let creator_address = api.addr_make("creator");
    auto_fee_manager::state::CREATOR_FEES.save(
        deps.as_mut().storage, 
        (&creator_address, "uusdc"), 
        &Uint128::from(500u128)
    ).unwrap();
    auto_fee_manager::state::CREATOR_FEES.save(
        deps.as_mut().storage, 
        (&creator_address, "uatom"), 
        &Uint128::from(300u128)
    ).unwrap();
    // Test successful claim
    let result = execute_claim_creator_fees(
        deps.as_mut(),
        env,
        creator_address.clone(),
    );
    assert!(result.is_ok());
    // Verify that creator fees were cleared
    let uusdc_fees = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator_address, "uusdc")
    ).unwrap();
    let uatom_fees = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator_address, "uatom")
    ).unwrap();
    assert_eq!(uusdc_fees, None);
    assert_eq!(uatom_fees, None);
    // Verify response attributes
    let response = result.unwrap();
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "claim_creator_fees");
    assert_eq!(response.attributes[1].key, "creator");
    assert_eq!(response.attributes[1].value, creator_address.to_string());
    // Verify bank messages were created
    assert_eq!(response.messages.len(), 2);
    // Check messages (order is alphabetical by denom: uatom, then uusdc)
    // First message (uatom)
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &response.messages[0].msg {
        assert_eq!(to_address, &creator_address.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, "uatom");
        assert_eq!(amount[0].amount, Uint128::from(300u128));
    } else {
        panic!("Expected BankMsg::Send");
    }
    // Second message (uusdc)
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &response.messages[1].msg {
        assert_eq!(to_address, &creator_address.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, "uusdc");
        assert_eq!(amount[0].amount, Uint128::from(500u128));
    } else {
        panic!("Expected BankMsg::Send");
    }
}
#[test]
fn test_claim_creator_fees_partial_claim() {
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
    // Add creator fees for multiple creators
    let creator1 = api.addr_make("creator1");
    let creator2 = api.addr_make("creator2");
    auto_fee_manager::state::CREATOR_FEES.save(
        deps.as_mut().storage, 
        (&creator1, "uusdc"), 
        &Uint128::from(500u128)
    ).unwrap();
    auto_fee_manager::state::CREATOR_FEES.save(
        deps.as_mut().storage, 
        (&creator2, "uusdc"), 
        &Uint128::from(300u128)
    ).unwrap();
    // Test claim by creator1
    let result = execute_claim_creator_fees(
        deps.as_mut(),
        env.clone(),
        creator1.clone(),
    );
    assert!(result.is_ok());
    // Verify that only creator1's fees were cleared
    let creator1_fees = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator1, "uusdc")
    ).unwrap();
    let creator2_fees = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator2, "uusdc")
    ).unwrap();
    assert_eq!(creator1_fees, None);
    assert_eq!(creator2_fees, Some(Uint128::from(300u128)));
    // Test claim by creator2
    let result = execute_claim_creator_fees(
        deps.as_mut(),
        env,
        creator2.clone(),
    );
    assert!(result.is_ok());
    // Verify that creator2's fees were also cleared
    let creator2_fees = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator2, "uusdc")
    ).unwrap();
    assert_eq!(creator2_fees, None);
} 