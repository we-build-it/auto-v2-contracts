use auto_fee_manager::msg::AcceptedDenom;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{Uint128};
use auto_fee_manager::ContractError;
use cosmwasm_std::{CosmosMsg, BankMsg};

mod utils;
use crate::utils::*;


#[test]
fn test_distribute_creator_fees_no_fees() {
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

    // Test distributing when no creator fees exist
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env,
        crank_authorized_address,
    );
    assert!(result.is_err());
    match result {
        Err(ContractError::NoCreatorFeesToDistribute {}) => {
            // Expected error
        },
        _ => panic!("Expected NoCreatorFeesToDistribute error"),
    }
}

#[test]
fn test_distribute_creator_fees_success() {
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

    // Add some creator fees
    let creator1 = api.addr_make("creator1");
    let creator2 = api.addr_make("creator2");
    
    // Subscribe creators to fee distribution
    auto_fee_manager::state::SUBSCRIBED_CREATORS.save(
        deps.as_mut().storage, 
        &creator1, 
        &true
    ).unwrap();
    auto_fee_manager::state::SUBSCRIBED_CREATORS.save(
        deps.as_mut().storage, 
        &creator2, 
        &true
    ).unwrap();
    
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
    auto_fee_manager::state::CREATOR_FEES.save(
        deps.as_mut().storage, 
        (&creator1, "uatom"), 
        &Uint128::from(200u128)
    ).unwrap();

    // Test successful distribution
    let result = execute_distribute_creator_fees(
        deps.as_mut(),
        env,
        crank_authorized_address,
    );
    assert!(result.is_ok());

    // Verify that creator fees were cleared
    let creator1_uusdc = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator1, "uusdc")
    ).unwrap();
    let creator2_uusdc = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator2, "uusdc")
    ).unwrap();
    let creator1_uatom = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator1, "uatom")
    ).unwrap();
    assert_eq!(creator1_uusdc, None);
    assert_eq!(creator2_uusdc, None);
    assert_eq!(creator1_uatom, None);

    // Verify that distribution fees were accumulated
    let distribution_fees_uusdc = auto_fee_manager::state::DISTRIBUTION_FEES.may_load(
        deps.as_ref().storage, 
        "uusdc"
    ).unwrap();
    let distribution_fees_uatom = auto_fee_manager::state::DISTRIBUTION_FEES.may_load(
        deps.as_ref().storage, 
        "uatom"
    ).unwrap();
    
    // 5% of 500 = 25, 5% of 300 = 15, 5% of 200 = 10
    assert_eq!(distribution_fees_uusdc, Some(Uint128::from(40u128))); // 25 + 15
    assert_eq!(distribution_fees_uatom, Some(Uint128::from(10u128))); // 10

    // Verify response attributes
    let response = result.unwrap();
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/distribute_creator_fees");
    assert_eq!(response.events[0].attributes.len(), 2);
    assert_eq!(response.events[0].attributes[0].key, "distribution_fee_rate");
    assert_eq!(response.events[0].attributes[0].value, "5");
    
    // Verify bank messages were created (should be 3: creator1 uusdc, creator1 uatom, creator2 uusdc)
    assert_eq!(response.messages.len(), 3);
    
    // Check that all messages are BankMsg::Send to the creators
    for message in response.messages {
        if let CosmosMsg::Bank(BankMsg::Send { to_address: _, amount }) = message.msg {
            assert_eq!(amount.len(), 1);
            let coin = &amount[0];
            
            // Verify amounts are correct (after 5% distribution fee)
            if coin.denom == "uusdc" {
                // creator1: 500 - 25 = 475, creator2: 300 - 15 = 285
                assert!(coin.amount == Uint128::from(475u128) || coin.amount == Uint128::from(285u128));
            } else if coin.denom == "uatom" {
                // creator1: 200 - 10 = 190
                assert_eq!(coin.amount, Uint128::from(190u128));
            } else {
                panic!("Unexpected denom: {}", coin.denom);
            }
        } else {
            panic!("Expected BankMsg::Send");
        }
    }
} 