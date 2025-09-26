use std::{collections::HashMap, str::FromStr};

use auto_workflow_manager::{msg::NewInstanceMsg};
use cosmwasm_std::{Addr, Decimal, Uint128};

mod utils;
use utils::{create_simple_test_workflow, create_test_environment, publish_workflow};

use auto_workflow_manager::{
    contract::{execute, query},
    msg::{ExecuteMsg, FeeTotal, FeeType, QueryMsg, UserFee},
    state::{PaymentConfig, PaymentSource},
};

use crate::utils::{create_oneshot_test_instance, execute_instance};

fn set_user_payment_config(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user_address: Addr,
    payment_config: PaymentConfig,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::SetUserPaymentConfig {
        payment_config,
    };
    let execute_info = cosmwasm_std::testing::message_info(&user_address, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn remove_user_payment_config(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user_address: Addr,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::RemoveUserPaymentConfig { };
    let execute_info = cosmwasm_std::testing::message_info(&user_address, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn query_user_payment_config(
    deps: &cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    user_address: String,
) -> Result<
    auto_workflow_manager::msg::GetUserPaymentConfigResponse,
    auto_workflow_manager::error::ContractError,
> {
    let query_msg = QueryMsg::GetUserPaymentConfig { user_address };
    let response = query(deps.as_ref(), cosmwasm_std::testing::mock_env(), query_msg)?;
    let result: auto_workflow_manager::msg::GetUserPaymentConfigResponse =
        cosmwasm_std::from_json(response)?;
    Ok(result)
}

fn create_test_payment_config() -> PaymentConfig {
    PaymentConfig {
        allowance: Uint128::new(1000),
        source: PaymentSource::Wallet,
    }
}

fn create_test_payment_config_prepaid() -> PaymentConfig {
    PaymentConfig {
        allowance: Uint128::new(500),
        source: PaymentSource::Prepaid,
    }
}

#[test]
fn test_set_user_payment_config_ok() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Set payment config
    let payment_config = create_test_payment_config();
    let response = set_user_payment_config(
        &mut deps,
        env,
        user_address.clone(),
        payment_config.clone(),
    )
    .unwrap();

    // Verify response events and attributes
    assert_eq!(response.attributes.len(), 0);
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-workflow-manager/set_user_payment_config");
    assert_eq!(response.events[0].attributes.len(), 3);
    assert_eq!(response.events[0].attributes[0].key, "user_address");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "allowance");
    assert_eq!(response.events[0].attributes[1].value, "1000");
    assert_eq!(response.events[0].attributes[2].key, "source");
    assert_eq!(response.events[0].attributes[2].value, "Wallet");

    // Verify that the payment config was actually saved in the state
    let saved_config = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(saved_config.payment_config.is_some());
    let config = saved_config.payment_config.unwrap();
    assert_eq!(config.allowance, Uint128::new(1000));
    assert!(matches!(config.source, PaymentSource::Wallet));
}

#[test]
fn test_query_user_payment_config_ok() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Set payment config first
    let payment_config = create_test_payment_config();
    set_user_payment_config(
        &mut deps,
        env,
        user_address.clone(),
        payment_config.clone(),
    )
    .unwrap();

    // Query the payment config
    let result = query_user_payment_config(&deps, user_address.to_string()).unwrap();

    // Verify the result
    assert!(result.payment_config.is_some());
    let config = result.payment_config.unwrap();
    assert_eq!(config.allowance, Uint128::new(1000));
    assert!(matches!(config.source, PaymentSource::Wallet));
}

#[test]
fn test_query_user_payment_config_not_found() {
    let (deps, _env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to query non-existent payment config
    let result = query_user_payment_config(&deps, user_address.to_string()).unwrap();

    // Verify that the operation returns None
    assert!(result.payment_config.is_none());
}

#[test]
fn test_remove_user_payment_config_ok() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Set payment config first
    let payment_config = create_test_payment_config();
    set_user_payment_config(
        &mut deps,
        env.clone(),
        user_address.clone(),
        payment_config,
    )
    .unwrap();

    // Verify it exists
    let query_result = query_user_payment_config(&deps, user_address.to_string());
    assert!(query_result.is_ok());

    // Remove the payment config
    let response =
        remove_user_payment_config(&mut deps, env, user_address.clone())
            .unwrap();

    // Verify response events and attributes
    assert_eq!(response.attributes.len(), 0);
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-workflow-manager/remove_user_payment_config");
    assert_eq!(response.events[0].attributes.len(), 1);
    assert_eq!(response.events[0].attributes[0].key, "user_address");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());

    // Verify it no longer exists
    let query_result = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(query_result.payment_config.is_none());
}

#[test]
fn test_payment_config_prepaid_source() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Set payment config with Prepaid source
    let payment_config = create_test_payment_config_prepaid();
    let response = set_user_payment_config(
        &mut deps,
        env.clone(),
        user_address.clone(),
        payment_config.clone(),
    )
    .unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 0);
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-workflow-manager/set_user_payment_config");
    assert_eq!(response.events[0].attributes.len(), 3);
    assert_eq!(response.events[0].attributes[0].key, "user_address");
    assert_eq!(response.events[0].attributes[0].value, user_address.to_string());
    assert_eq!(response.events[0].attributes[1].key, "allowance");
    assert_eq!(response.events[0].attributes[1].value, "500");
    assert_eq!(response.events[0].attributes[2].key, "source");
    assert_eq!(response.events[0].attributes[2].value, "Prepaid");

    // Query and verify the config
    let result = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(result.payment_config.is_some());
    let config = result.payment_config.unwrap();
    assert_eq!(config.allowance, Uint128::new(500));
    assert!(matches!(config.source, PaymentSource::Prepaid));

    // Verify that the payment config was actually saved in the state
    let saved_config = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(saved_config.payment_config.is_some());
    let saved_config_unwrapped = saved_config.payment_config.unwrap();
    assert_eq!(saved_config_unwrapped.allowance, Uint128::new(500));
    assert!(matches!(
        saved_config_unwrapped.source,
        PaymentSource::Prepaid
    ));
}

#[test]
fn test_update_user_payment_config() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Set initial payment config
    let initial_config = create_test_payment_config();
    set_user_payment_config(
        &mut deps,
        env.clone(),
        user_address.clone(),
        initial_config,
    )
    .unwrap();

    // Update with new config
    let updated_config = PaymentConfig {
        allowance: Uint128::new(2000),
        source: PaymentSource::Prepaid,
    };
    set_user_payment_config(
        &mut deps,
        env,
        user_address.clone(),
        updated_config.clone(),
    )
    .unwrap();

    // Query and verify the updated config
    let result = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(result.payment_config.is_some());
    let config = result.payment_config.unwrap();
    assert_eq!(config.allowance, Uint128::new(2000));
    assert!(matches!(config.source, PaymentSource::Prepaid));

    // Verify that the payment config was actually updated in the state
    let saved_config = query_user_payment_config(&deps, user_address.to_string()).unwrap();
    assert!(saved_config.payment_config.is_some());
    let saved_config_unwrapped = saved_config.payment_config.unwrap();
    assert_eq!(saved_config_unwrapped.allowance, Uint128::new(2000));
    assert!(matches!(
        saved_config_unwrapped.source,
        PaymentSource::Prepaid
    ));
}

#[test]
fn test_charge_fees_events() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) =
        create_test_environment();

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Create users
    let user1: Addr = api.addr_make("user1");
    let user2: Addr = api.addr_make("user2");

    // Set payment config
    let payment_config = create_test_payment_config_prepaid();
    set_user_payment_config(
        &mut deps,
        env.clone(),
        user1.clone(),
        payment_config.clone(),
    )
    .unwrap();
    set_user_payment_config(
        &mut deps,
        env.clone(),
        user2.clone(),
        payment_config.clone(),
    )
    .unwrap();

    // Prices to use for the fees
    let prices = HashMap::from([
        ("RUNE".to_string(), Decimal::from_str("0.5").unwrap()),
        ("AUTO".to_string(), Decimal::from_str("0.5").unwrap()),
        ("TCY".to_string(), Decimal::from_str("0.5").unwrap()),
        ("uusdc".to_string(), Decimal::from_str("0.5").unwrap()),
    ]);

    // Execute instance
    let instance1: NewInstanceMsg =
        create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance1).unwrap();
    let instance2: NewInstanceMsg =
        create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user2.clone(), instance2).unwrap();

    // Create test fees
    let fees = vec![
        UserFee {
            address: user1.to_string(),
            totals: vec![
                FeeTotal {
                    denom: "RUNE".to_string(),
                    amount: Uint128::new(100000000),
                    fee_type: FeeType::Execution,
                },
                FeeTotal {
                    denom: "AUTO".to_string(),
                    amount: Uint128::new(10000000000),
                    fee_type: FeeType::Creator { instance_id: 1 },
                },
            ],
        },
        UserFee {
            address: user2.to_string(),
            totals: vec![FeeTotal {
                denom: "TCY".to_string(),
                amount: Uint128::new(1000000000000),
                fee_type: FeeType::Execution,
            }],
        },
    ];

    let batch_id = "test-batch-123".to_string();

    // Execute charge_fees
    let response = charge_fees(&mut deps, env, admin_address, batch_id.clone(), prices, fees).unwrap();

    // Verify response events and attributes
    assert_eq!(response.attributes.len(), 0);
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-workflow-manager/charge_fees");
    assert_eq!(response.events[0].attributes.len(), 1);
    assert_eq!(response.events[0].attributes[0].key, "batch_id");
    assert_eq!(response.events[0].attributes[0].value, batch_id);

    // Verify that submessages were created correctly
    assert_eq!(response.messages.len(), 2); // 2 submessages for 2 users (one per user)
    
    // Note: In a real blockchain environment, these submessages would trigger replies
    // and the reply function would emit fee-charged events. In tests, we can't easily
    // simulate this, but we can verify that the data was stored correctly and that
    // the reply function exists and works.
    
    // For now, we'll test the reply function separately in a dedicated test
}

#[test]
fn test_handle_fee_manager_reply() {
    let (mut deps, env, _api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    
    // Create test fee event data
    let fee_event_data = auto_workflow_manager::execute::FeeEventData {
        user_address: "thor1test".to_string(),
        original_denom: "RUNE".to_string(),
        original_amount_charged: Uint128::new(1000),
        discounted_from_allowance: Uint128::new(1000),
        debit_denom: "RUNE".to_string(),
        creator_address: Some("thor1test".to_string()),
        fee_type: FeeType::Execution,
    };
    
    // Store the data in the temporary storage
    use auto_workflow_manager::execute::FEE_EVENT_DATA;
    FEE_EVENT_DATA.save(deps.as_mut().storage, 1, &vec![fee_event_data]).unwrap();
    
    // Create a mock reply
    #[allow(deprecated)]
    let reply = cosmwasm_std::Reply {
        id: 1,
        result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
            events: vec![],
            msg_responses: vec![],
            data: None,
            // msg_responses: vec![],
        }),
        gas_used: 0,
        payload: cosmwasm_std::Binary::default(),
    };
    
    // Call the reply function directly
    let response = auto_workflow_manager::execute::handle_fee_manager_reply(
        deps.as_mut(),
        env,
        reply,
    ).unwrap();
    
    // Verify the response (no attributes, only events)
    assert_eq!(response.attributes.len(), 0);
    
    // Verify the fee-charged event
    assert_eq!(response.events.len(), 1);
    let fee_event = &response.events[0];
    assert_eq!(fee_event.ty, "autorujira-workflow-manager/fee-charged");
    assert_eq!(fee_event.attributes.len(), 7);
    
    // Check specific attributes
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "user_address" && attr.value == "thor1test"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "original_denom" && attr.value == "RUNE"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "original_amount_charged" && attr.value == "1000"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "discounted_from_allowance" && attr.value == "1000"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "debit_denom" && attr.value == "RUNE"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "fee_type" && attr.value == "execution"));
    assert!(fee_event.attributes.iter().any(|attr| attr.key == "creator_address" && attr.value == "thor1test"));
}

fn charge_fees(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    admin: Addr,
    batch_id: String,
    prices: HashMap<String, Decimal>,
    fees: Vec<UserFee>,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ChargeFees { batch_id, prices, fees };
    let execute_info = cosmwasm_std::testing::message_info(&admin, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}
