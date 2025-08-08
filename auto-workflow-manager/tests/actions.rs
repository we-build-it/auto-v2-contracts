use auto_workflow_manager::{
    contract::execute,
    msg::{ActionParamValue, ExecuteMsg, NewInstanceMsg, ExecutionType},
};
use cosmwasm_std::{Addr, Timestamp};
use std::collections::HashMap;
use std::collections::HashSet;

mod utils;
use utils::{create_simple_test_workflow, publish_workflow, create_test_environment};

use auto_workflow_manager::{
    error::ContractError,
};

fn create_test_instance(workflow_id: String) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: HashMap::new(),
        execution_type: ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(1000000000), // Far future
    }
}

fn create_test_instance_with_expiration(workflow_id: String, expiration_time: Timestamp) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: HashMap::new(),
        execution_type: ExecutionType::OneShot,
        expiration_time,
    }
}

fn execute_instance(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::testing::MockStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier, cosmwasm_std::Empty>,
    env: cosmwasm_std::Env,
    user: Addr,
    instance: NewInstanceMsg,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ExecuteInstance { instance };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn execute_action(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::testing::MockStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier, cosmwasm_std::Empty>,
    env: cosmwasm_std::Env,
    executor: Addr,
    user_address: String,
    instance_id: u64,
    action_id: String,
    template_id: String,
    params: Option<HashMap<String, ActionParamValue>>,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ExecuteAction {
        user_address,
        instance_id,
        action_id,
        template_id,
        params,
    };
    let execute_info = cosmwasm_std::testing::message_info(&executor, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

#[test]
fn test_execute_action_ok() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with correct parameters for token_staker
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Execute action
    let response = execute_action(
        &mut deps,
        env,
        executor_address.clone(),
        user_address.to_string(),
        1,
        "stake_tokens".to_string(),
        "default".to_string(),
        None,
    ).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "execute_action");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user_address.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "action_id");
    assert_eq!(response.attributes[3].value, "stake_tokens");
}

#[test]
fn test_execute_action_unauthorized_executor() {
    let (mut deps, mut env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");
    let unauthorized_executor = api.addr_make("unauthorized_executor");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with correct parameters for token_staker
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to execute action with unauthorized executor
    let result = execute_action(
        &mut deps,
        env,
        unauthorized_executor,
        user_address.to_string(),
        1,
        "stake_tokens".to_string(),
        "default".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::Unauthorized { .. }) => {
            // Expected error
        }
        _ => panic!("Expected Unauthorized error, got different error"),
    }
}

#[test]
fn test_execute_action_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Try to execute action on non-existent instance
    let result = execute_action(
        &mut deps,
        env,
        executor_address,
        user_address.to_string(),
        999,
        "stake_tokens".to_string(),
        "default".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    // The function doesn't handle InstanceNotFound error properly, so we expect a Std error
    match result {
        Err(ContractError::Std(_)) => {
            // Expected error - the function doesn't convert NotFound to InstanceNotFound
        }
        _ => panic!("Expected Std error, got different error: {:?}", result),
    }
}

#[test]
fn test_execute_action_instance_expired() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set block time to future to make instance expire
    env.block.time = Timestamp::from_seconds(2000000000);

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to execute action on expired instance
    let result = execute_action(
        &mut deps,
        env,
        executor_address,
        user_address.to_string(),
        1,
        "stake_tokens".to_string(),
        "default".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Instance has expired");
        }
        _ => panic!("Expected GenericError with 'Instance has expired', got different error"),
    }
}

#[test]
fn test_execute_action_action_not_found() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with correct parameters for token_staker
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to execute non-existent action
    let result = execute_action(
        &mut deps,
        env,
        executor_address,
        user_address.to_string(),
        1,
        "non_existent_action".to_string(),
        "default".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::ActionNotFound { workflow_id, action_id }) => {
            assert_eq!(workflow_id, "simple-test-workflow");
            assert_eq!(action_id, "non_existent_action");
        }
        _ => panic!("Expected ActionNotFound error, got different error: {:?}", result),
    }
}

#[test]
fn test_execute_action_invalid_action_sequence() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with multiple actions
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);
    
    workflow_msg.actions.insert(
        "second_action".to_string(),
        auto_workflow_manager::msg::ActionMsg {
            params: HashMap::from([
                ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
                ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
                ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
                ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
                ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
            ]),
            next_actions: std::collections::HashSet::new(),
            templates: HashMap::from([
                (
                    "default".to_string(),
                    auto_workflow_manager::msg::Template {
                        contract: "{{contractAddress}}".to_string(),
                        message: "{\"stake\":{ \"amount\": {{amount}} }}".to_string(),
                        funds: vec![],
                    },
                ),
            ]),
            whitelisted_contracts: HashSet::from([
                "osmo1contract123456789".to_string(),
            ]),
        },
    );
    // Update the first action to have next_actions
    if let Some(first_action) = workflow_msg.actions.get_mut("stake_tokens") {
        first_action.next_actions.insert("second_action".to_string());
    }

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to execute second action without executing first action
    let result = execute_action(
        &mut deps,
        env,
        executor_address,
        user_address.to_string(),
        1,
        "second_action".to_string(),
        "default".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::GenericError(message)) => {
            assert!(message.contains("Action cannot be executed"));
        }
        _ => panic!("Expected GenericError with action execution message, got different error: {:?}", result),
    }
}

#[test]
fn test_execute_action_with_params() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with correct parameters for token_staker
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Execute action with additional parameters
    let mut params = HashMap::new();
    params.insert("extra_param".to_string(), ActionParamValue::String("extra_value".to_string()));

    let response = execute_action(
        &mut deps,
        env,
        executor_address.clone(),
        user_address.to_string(),
        1,
        "stake_tokens".to_string(),
        "default".to_string(),
        Some(params),
    ).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "execute_action");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user_address.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "action_id");
    assert_eq!(response.attributes[3].value, "stake_tokens");
}

#[test]
fn test_execute_action_recurrent_workflow() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with correct parameters for token_staker
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.actions.get_mut("stake_tokens").unwrap().params = HashMap::from([
        ("provider".to_string(), ActionParamValue::String("daodao".to_string())),
        ("contractAddress".to_string(), ActionParamValue::String("osmo1contract123456789".to_string())),
        ("userAddress".to_string(), ActionParamValue::String(user_address.to_string())),
        ("amount".to_string(), ActionParamValue::BigInt("1000000".to_string())),
        ("denom".to_string(), ActionParamValue::String("uosmo".to_string())),
    ]);

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with recurrent execution type and future expiration
    let mut instance = create_test_instance_with_expiration("simple-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    instance.execution_type = ExecutionType::Recurrent;
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Execute the start action
    let response = execute_action(
        &mut deps,
        env,
        executor_address.clone(),
        user_address.to_string(),
        1,
        "stake_tokens".to_string(),
        "default".to_string(),
        None,
    ).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "execute_action");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user_address.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "action_id");
    assert_eq!(response.attributes[3].value, "stake_tokens");
} 

#[test]
fn test_execute_action_with_dynamic_template() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with dynamic templates
    let workflow_msg = auto_workflow_manager::msg::NewWorkflowMsg {
        id: "template-test-workflow".to_string(),
        start_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
        visibility: auto_workflow_manager::msg::WorkflowVisibility::Public,
        actions: HashMap::from([
            (
                "claim_tokens".to_string(),
                auto_workflow_manager::msg::ActionMsg {
                    params: HashMap::from([
                        (
                            "contractAddress".to_string(),
                            ActionParamValue::String("osmo1contract123456789abcdefghijklmnopqrstuvwxyz".to_string()),
                        ),
                        (
                            "distributionId".to_string(),
                            ActionParamValue::String("123".to_string()),
                        ),
                    ]),
                    next_actions: HashSet::new(),
                    templates: HashMap::from([
                        (
                            "daodao".to_string(),
                            auto_workflow_manager::msg::Template {
                                contract: "{{contractAddress}}".to_string(),
                                message: "{\"claim\":{ \"id\": {{distributionId}} }}".to_string(),
                                funds: vec![],
                            },
                        ),
                        (
                            "rujira".to_string(),
                            auto_workflow_manager::msg::Template {
                                contract: "{{contractAddress}}".to_string(),
                                message: "{\"claim\":{ \"otherId\": {{distributionId}} }}".to_string(),
                                funds: vec![],
                            },
                        ),
                    ]),
                    whitelisted_contracts: HashSet::from([
                        "osmo1contract123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                    ]),
                },
            ),
        ]),
    };

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("template-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Execute action with daodao template
    let response = execute_action(
        &mut deps,
        env.clone(),
        executor_address.clone(),
        user_address.to_string(),
        1,
        "claim_tokens".to_string(),
        "daodao".to_string(),
        None,
    ).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "execute_action");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user_address.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "action_id");
    assert_eq!(response.attributes[3].value, "claim_tokens");

    // Verify that sub-messages were created
    assert_eq!(response.messages.len(), 1);
}

#[test]
fn test_execute_action_template_not_found() {
    let (mut deps, mut env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Set a reasonable block time
    env.block.time = Timestamp::from_seconds(1000000000);

    // Create a workflow with dynamic templates
    let workflow_msg = auto_workflow_manager::msg::NewWorkflowMsg {
        id: "template-test-workflow".to_string(),
        start_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
        visibility: auto_workflow_manager::msg::WorkflowVisibility::Public,
        actions: HashMap::from([
            (
                "claim_tokens".to_string(),
                auto_workflow_manager::msg::ActionMsg {
                    params: HashMap::from([
                        (
                            "contractAddress".to_string(),
                            ActionParamValue::String("osmo1contract123456789abcdefghijklmnopqrstuvwxyz".to_string()),
                        ),
                        (
                            "distributionId".to_string(),
                            ActionParamValue::String("123".to_string()),
                        ),
                    ]),
                    next_actions: HashSet::new(),
                    templates: HashMap::from([
                        (
                            "daodao".to_string(),
                            auto_workflow_manager::msg::Template {
                                contract: "{{contractAddress}}".to_string(),
                                message: "{\"claim\":{ \"id\": {{distributionId}} }}".to_string(),
                                funds: vec![],
                            },
                        ),
                    ]),
                    whitelisted_contracts: HashSet::from([
                        "osmo1contract123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                    ]),
                },
            ),
        ]),
    };

    // Publish the workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance with future expiration
    let instance = create_test_instance_with_expiration("template-test-workflow".to_string(), Timestamp::from_seconds(2000000000));
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to execute action with non-existent template
    let result = execute_action(
        &mut deps,
        env,
        executor_address,
        user_address.to_string(),
        1,
        "claim_tokens".to_string(),
        "non_existent_template".to_string(),
        None,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::TemplateNotFound { workflow_id, action_id, template_id }) => {
            assert_eq!(workflow_id, "template-test-workflow");
            assert_eq!(action_id, "claim_tokens");
            assert_eq!(template_id, "non_existent_template");
        }
        _ => panic!("Expected TemplateNotFound error, got different error: {:?}", result),
    }
} 