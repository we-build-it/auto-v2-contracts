use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
use cosmwasm_std::{Uint128, CosmosMsg, BankMsg};
use auto_fee_manager::contract::{execute, query};
use auto_fee_manager::msg::{AcceptedDenom, ExecuteMsg, QueryMsg};

mod utils;
use crate::utils::*;

#[test]
fn test_enable_creator_fee_distribution() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_dest");
    let distribution_fees_destination_address = api.addr_make("distribution_dest");
    let crank_authorized_address = api.addr_make("crank_auth");
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
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5%
    ).unwrap();

    // Test enabling creator fee distribution
    let creator = api.addr_make("creator1");
    let result = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator, &[]),
        ExecuteMsg::EnableCreatorFeeDistribution {},
    );
    
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/enable_creator_fee_distribution");
    assert_eq!(response.events[0].attributes.len(), 1);
    assert_eq!(response.events[0].attributes[0].key, "creator");
    assert_eq!(response.events[0].attributes[0].value, creator.to_string());
}

#[test]
fn test_disable_creator_fee_distribution() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_dest");
    let distribution_fees_destination_address = api.addr_make("distribution_dest");
    let crank_authorized_address = api.addr_make("crank_auth");
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
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5%
    ).unwrap();

    // Test disabling creator fee distribution
    let creator = api.addr_make("creator1");
    let result = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator, &[]),
        ExecuteMsg::DisableCreatorFeeDistribution {},
    );
    
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-fee-manager/disable_creator_fee_distribution");
    assert_eq!(response.events[0].attributes.len(), 1);
    assert_eq!(response.events[0].attributes[0].key, "creator");
    assert_eq!(response.events[0].attributes[0].value, creator.to_string());
}

#[test]
fn test_is_creator_subscribed() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_dest");
    let distribution_fees_destination_address = api.addr_make("distribution_dest");
    let crank_authorized_address = api.addr_make("crank_auth");
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
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5%
    ).unwrap();

    let creator1 = api.addr_make("creator1");
    let creator2 = api.addr_make("creator2");
    
    // Initially, creators should not be subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::IsCreatorSubscribed { creator: creator1.clone() },
    );
    assert!(result.is_ok());
    let is_subscribed: bool = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(is_subscribed, false);
    
    // Subscribe creator1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator1, &[]),
        ExecuteMsg::EnableCreatorFeeDistribution {},
    ).unwrap();
    
    // Check that creator1 is now subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::IsCreatorSubscribed { creator: creator1.clone() },
    );
    assert!(result.is_ok());
    let is_subscribed: bool = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(is_subscribed, true);
    
    // Check that creator2 is still not subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::IsCreatorSubscribed { creator: creator2.clone() },
    );
    assert!(result.is_ok());
    let is_subscribed: bool = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(is_subscribed, false);
    
    // Unsubscribe creator1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator1, &[]),
        ExecuteMsg::DisableCreatorFeeDistribution {},
    ).unwrap();
    
    // Check that creator1 is now unsubscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::IsCreatorSubscribed { creator: creator1.clone() },
    );
    assert!(result.is_ok());
    let is_subscribed: bool = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(is_subscribed, false);
}

#[test]
fn test_get_subscribed_creators() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_dest");
    let distribution_fees_destination_address = api.addr_make("distribution_dest");
    let crank_authorized_address = api.addr_make("crank_auth");
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
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address,
        workflow_manager_address,
        Uint128::from(5u128), // 5%
    ).unwrap();

    let creator1 = api.addr_make("creator1");
    let creator2 = api.addr_make("creator2");
    let creator3 = api.addr_make("creator3");
    
    // Initially, no creators should be subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetSubscribedCreators {},
    );
    assert!(result.is_ok());
    let subscribed_creators: auto_fee_manager::msg::SubscribedCreatorsResponse = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(subscribed_creators.creators.len(), 0);
    
    // Subscribe creator1 and creator2
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator1, &[]),
        ExecuteMsg::EnableCreatorFeeDistribution {},
    ).unwrap();
    
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator2, &[]),
        ExecuteMsg::EnableCreatorFeeDistribution {},
    ).unwrap();
    
    // Check that both creators are subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetSubscribedCreators {},
    );
    assert!(result.is_ok());
    let subscribed_creators: auto_fee_manager::msg::SubscribedCreatorsResponse = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(subscribed_creators.creators.len(), 2);
    assert!(subscribed_creators.creators.contains(&creator1));
    assert!(subscribed_creators.creators.contains(&creator2));
    assert!(!subscribed_creators.creators.contains(&creator3));
    
    // Unsubscribe creator1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator1, &[]),
        ExecuteMsg::DisableCreatorFeeDistribution {},
    ).unwrap();
    
    // Check that only creator2 is subscribed
    let result = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetSubscribedCreators {},
    );
    assert!(result.is_ok());
    let subscribed_creators: auto_fee_manager::msg::SubscribedCreatorsResponse = cosmwasm_std::from_json(result.unwrap()).unwrap();
    assert_eq!(subscribed_creators.creators.len(), 1);
    assert!(!subscribed_creators.creators.contains(&creator1));
    assert!(subscribed_creators.creators.contains(&creator2));
}

#[test]
fn test_distribute_creator_fees_only_subscribed() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;
    
    // Setup contract
    let admin_address = api.addr_make("admin");
    let execution_fees_destination_address = api.addr_make("execution_dest");
    let distribution_fees_destination_address = api.addr_make("distribution_dest");
    let crank_authorized_address = api.addr_make("crank_auth");
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
        execution_fees_destination_address,
        distribution_fees_destination_address,
        crank_authorized_address.clone(),
        workflow_manager_address,
        Uint128::from(5u128), // 5%
    ).unwrap();

    let creator1 = api.addr_make("creator1");
    let creator2 = api.addr_make("creator2");
    
    // Subscribe only creator1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&creator1, &[]),
        ExecuteMsg::EnableCreatorFeeDistribution {},
    ).unwrap();
    
    // Add creator fees for both creators
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
    
    // Distribute creator fees
    let result = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&crank_authorized_address, &[]),
        ExecuteMsg::DistributeCreatorFees {},
    );
    
    assert!(result.is_ok());
    
    let response = result.unwrap();
    
    // Should only have 1 bank message (for creator1, not creator2)
    assert_eq!(response.messages.len(), 1);
    
    // Check that creator1's fees were distributed
    let creator1_balance = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator1, "uusdc")
    ).unwrap();
    assert_eq!(creator1_balance, None); // Should be cleared
    
    // Check that creator2's fees were NOT distributed (because not subscribed)
    let creator2_balance = auto_fee_manager::state::CREATOR_FEES.may_load(
        deps.as_ref().storage, 
        (&creator2, "uusdc")
    ).unwrap();
    assert_eq!(creator2_balance, Some(Uint128::from(300u128))); // Should still be there
    
    // Verify the bank message is for creator1
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &response.messages[0].msg {
        assert_eq!(to_address, &creator1.to_string());
        assert_eq!(amount.len(), 1);
        let coin = &amount[0];
        assert_eq!(coin.denom, "uusdc");
        // 500 - 5% = 475
        assert_eq!(coin.amount, Uint128::from(475u128));
    } else {
        panic!("Expected BankMsg::Send");
    }
} 