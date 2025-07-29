use auto_workflow_manager::ContractError;
use cosmwasm_std::{Addr, Timestamp};

mod utils;
use utils::{create_simple_test_workflow, publish_workflow, create_test_environment};

use auto_workflow_manager::{
    contract::execute,
    msg::{ExecuteMsg, NewInstanceMsg, ExecutionType},
};

fn create_test_instance(workflow_id: String) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: std::collections::HashMap::new(),
        execution_type: ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(1000000000), // Far future
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

fn cancel_instance(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::testing::MockStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier, cosmwasm_std::Empty>,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::CancelInstance { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn pause_instance(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::testing::MockStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier, cosmwasm_std::Empty>,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::PauseInstance { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn resume_instance(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::testing::MockStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier, cosmwasm_std::Empty>,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ResumeInstance { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

#[test]
fn test_execute_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    let response = execute_instance(&mut deps, env, user_address.clone(), instance).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "execute_instance");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1"); // First instance should have ID 1
    assert_eq!(response.attributes[2].key, "workflow_id");
    assert_eq!(response.attributes[2].value, "simple-test-workflow");
    assert_eq!(response.attributes[3].key, "requester");
    assert_eq!(response.attributes[3].value, user_address.to_string());
}

#[test]
fn test_execute_instance_workflow_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Try to execute instance with non-existent workflow
    let instance = create_test_instance("non-existent-workflow".to_string());
    let result = execute_instance(&mut deps, env, user_address, instance);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::WorkflowNotFound { workflow_id }) => {
            assert_eq!(workflow_id, "non-existent-workflow");
        }
        _ => panic!("Expected WorkflowNotFound error, got different error"),
    }
}

#[test]
fn test_execute_instance_private_workflow_denied() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let unauthorized_user = api.addr_make("unauthorized_user");

    // Create and publish a private workflow
    let mut workflow_msg = create_simple_test_workflow();
    workflow_msg.visibility = auto_workflow_manager::msg::WorkflowVisibility::Private;
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Try to execute instance with unauthorized user
    let instance = create_test_instance("simple-test-workflow".to_string());
    let result = execute_instance(&mut deps, env, unauthorized_user, instance);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::PrivateWorkflowExecutionDenied { workflow_id }) => {
            assert_eq!(workflow_id, "simple-test-workflow");
        }
        _ => panic!("Expected PrivateWorkflowExecutionDenied error, got different error"),
    }
}

#[test]
fn test_cancel_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Cancel the instance
    let response = cancel_instance(&mut deps, env, user_address.clone(), 1).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "cancel_instance");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "canceller");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

#[test]
fn test_cancel_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Try to cancel non-existent instance
    let result = cancel_instance(&mut deps, env, user_address, 999);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::InstanceNotFound { instance_id }) => {
            assert_eq!(instance_id, "999");
        }
        _ => panic!("Expected InstanceNotFound error, got different error"),
    }
}

#[test]
fn test_pause_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance
    let response = pause_instance(&mut deps, env, user_address.clone(), 1).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "pause_instance");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "pauser");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

#[test]
fn test_pause_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Try to pause non-existent instance
    let result = pause_instance(&mut deps, env, user_address, 999);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::InstanceNotFound { instance_id }) => {
            assert_eq!(instance_id, "999");
        }
        _ => panic!("Expected InstanceNotFound error, got different error"),
    }
}

#[test]
fn test_pause_instance_already_paused() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance first time
    pause_instance(&mut deps, env.clone(), user_address.clone(), 1).unwrap();

    // Try to pause the instance again
    let result = pause_instance(&mut deps, env, user_address, 1);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Instance is not running");
        }
        _ => panic!("Expected GenericError with 'Instance is not running', got different error"),
    }
}

#[test]
fn test_resume_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance
    pause_instance(&mut deps, env.clone(), user_address.clone(), 1).unwrap();

    // Resume the instance
    let response = resume_instance(&mut deps, env, user_address.clone(), 1).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "resume_instance");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "resumer");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

#[test]
fn test_resume_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Try to resume non-existent instance
    let result = resume_instance(&mut deps, env, user_address, 999);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::InstanceNotFound { instance_id }) => {
            assert_eq!(instance_id, "999");
        }
        _ => panic!("Expected InstanceNotFound error, got different error"),
    }
}

#[test]
fn test_resume_instance_not_paused() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow();
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Execute instance
    let instance = create_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to resume running instance
    let result = resume_instance(&mut deps, env, user_address, 1);

    // Verify that the operation fails
    assert!(result.is_err());
    
    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Instance is not paused");
        }
        _ => panic!("Expected GenericError with 'Instance is not paused', got different error"),
    }
} 