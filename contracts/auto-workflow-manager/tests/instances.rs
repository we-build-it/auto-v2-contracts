use auto_workflow_manager::ContractError;
use cosmwasm_std::{Addr, Timestamp};

mod utils;
use utils::{create_simple_test_workflow, create_test_environment, publish_workflow};

use auto_workflow_manager::{
    contract::execute,
    msg::{ExecuteMsg, ExecutionType, NewInstanceMsg, WorkflowInstanceState},
    query::query_workflow_instance,
};

fn create_oneshot_test_instance(workflow_id: String) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: std::collections::HashMap::new(),
        offchain_parameters: std::collections::HashMap::new(),
        execution_type: ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(1000000000), // Far future
        cron_expression: None,
    }
}

fn create_recurrent_test_instance(workflow_id: String) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: std::collections::HashMap::new(),
        offchain_parameters: std::collections::HashMap::new(),
        execution_type: ExecutionType::Recurrent,
        expiration_time: Timestamp::from_seconds(1000000000), // Far future
        cron_expression: None,
    }
}

fn execute_instance(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user: Addr,
    instance: NewInstanceMsg,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ExecuteInstance { instance };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn cancel_run(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::CancelRun {
        instance_id
    };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn cancel_instance(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::CancelInstance { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn pause_schedule(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::PauseSchedule { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

fn resume_schedule(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    user: Addr,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ResumeSchedule { instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&user, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

// ----------------------------------------------------------------------------------------
// -------------------------------- execute instance tests --------------------------------
// ----------------------------------------------------------------------------------------

#[test]
fn test_execute_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
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
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to execute instance with non-existent workflow
    let instance = create_oneshot_test_instance("non-existent-workflow".to_string());
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
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let unauthorized_user = api.addr_make("unauthorized_user");

    // Create and publish a private workflow
    let mut workflow_msg = create_simple_test_workflow(api);
    workflow_msg.visibility = auto_workflow_manager::msg::WorkflowVisibility::Private;
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Try to execute instance with unauthorized user
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
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

// ----------------------------------------------------------------------------------
// -------------------------------- cancel run tests --------------------------------
// ----------------------------------------------------------------------------------

#[test]
fn test_cancel_instance_run_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to cancel non-existent instance
    let result = cancel_run(&mut deps, env, user_address, 999);

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
fn test_cancel_instance_oneshot_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Cancel the instance
    let response = cancel_instance(
        &mut deps,
        env,
        user_address.clone(),
        1
    )
    .unwrap();

    // Check that the instance no longer exists in the contract state
    let instance_query = query_workflow_instance(deps.as_ref(), user_address.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Cancelled);

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
fn test_cancel_instance_recurrent_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Cancel the instance
    let response = cancel_instance(
        &mut deps,
        env,
        user_address.clone(),
        1
    )
    .unwrap();

    // Check that the instance exists in the contract state
    let instance_query = query_workflow_instance(deps.as_ref(), user_address.to_string(), 1);
    assert!(instance_query.is_ok());

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "cancel_instance");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "canceller");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

// ---------------------------------------------------------------------------------------
// -------------------------------- cancel schedule tests --------------------------------
// ---------------------------------------------------------------------------------------

#[test]
fn test_cancel_run_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to cancel non-existent instance
    let result = cancel_run(&mut deps, env, user_address, 999);

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
fn test_cancel_run_oneshot_fail() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Cancel the instance
    let result = cancel_run(&mut deps, env, user_address.clone(), 1);
    assert!(result.is_err());

    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Can't cancel run for one shot instances, use cancel_instance instead");
        }
        _ => panic!("Expected GenericError with 'Can't cancel run for one shot instances, use cancel_instance instead', got different error"),
    }
}

#[test]
fn test_cancel_run_recurrent_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Cancel the instance
    let result = cancel_run(&mut deps, env, user_address.clone(), 1);
    assert!(result.is_err());

    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Can't cancel run for one shot instances, use cancel_instance instead");
        }
        _ => panic!("Expected GenericError with 'Can't cancel run for one shot instances, use cancel_instance instead', got different error"),
    }
}

// --------------------------------------------------------------------------------------
// -------------------------------- pause schedule tests --------------------------------
// --------------------------------------------------------------------------------------

#[test]
fn test_pause_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to pause non-existent instance
    let result = pause_schedule(&mut deps, env, user_address, 999);

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
fn test_pause_recurrent_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance
    let response = pause_schedule(&mut deps, env, user_address.clone(), 1).unwrap();

    // Check that the instance state is paused
    let instance_query = query_workflow_instance(deps.as_ref(), user_address.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Paused);

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "pause_schedule");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "pauser");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

#[test]
fn test_pause_oneshot_instance_fail() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance
    let result = pause_schedule(&mut deps, env, user_address.clone(), 1);
    assert!(result.is_err());

    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Can't change schedule for non-recurrent instances");
        }
        _ => panic!("Expected GenericError with 'Can't change schedule for non-recurrent instances', got different error"),
    }
}

#[test]
fn test_pause_instance_already_paused() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance first time
    pause_schedule(&mut deps, env.clone(), user_address.clone(), 1).unwrap();

    // Try to pause the instance again
    let result = pause_schedule(&mut deps, env, user_address, 1);

    // Verify that the operation fails
    assert!(result.is_err());

    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Instance is not running");
        }
        _ => panic!("Expected GenericError with 'Instance is not running', got different error"),
    }
}

// ---------------------------------------------------------------------------------------
// -------------------------------- resume schedule tests --------------------------------
// ---------------------------------------------------------------------------------------
#[test]
fn test_resume_instance_not_found() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Try to resume non-existent instance
    let result = resume_schedule(&mut deps, env, user_address, 999);

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
fn test_resume_oneshot_instance_fail() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Resume the instance
    let result = resume_schedule(&mut deps, env, user_address.clone(), 1);
    assert!(result.is_err());
    
    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Can't change schedule for non-recurrent instances");
        }
        _ => panic!("Expected GenericError with 'Can't change schedule for non-recurrent instances', got different error"),
    }
}

#[test]
fn test_resume_recurrent_instance_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Pause the instance
    pause_schedule(&mut deps, env.clone(), user_address.clone(), 1).unwrap();

    // Resume the instance
    let response = resume_schedule(&mut deps, env, user_address.clone(), 1).unwrap();

    // Check that the instance state is running
    let instance_query = query_workflow_instance(deps.as_ref(), user_address.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Running);
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "resume_schedule");
    assert_eq!(response.attributes[1].key, "instance_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "resumer");
    assert_eq!(response.attributes[2].value, user_address.to_string());
}

#[test]
fn test_resume_instance_not_paused() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) =
        create_test_environment();
    let user_address = api.addr_make("user");

    // Publish a workflow first
    let workflow_msg = create_simple_test_workflow(api);
    publish_workflow(
        deps.as_mut(),
        env.clone(),
        publisher_address.clone(),
        workflow_msg,
    )
    .unwrap();

    // Execute instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user_address.clone(), instance).unwrap();

    // Try to resume running instance
    let result = resume_schedule(&mut deps, env, user_address, 1);

    // Verify that the operation fails
    assert!(result.is_err());

    match result {
        Err(ContractError::GenericError(message)) => {
            assert_eq!(message, "Instance is not paused");
        }
        _ => panic!("Expected GenericError with 'Instance is not paused', got different error"),
    }
}
