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

fn reset_instance(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    admin: Addr,
    user_address: String,
    instance_id: u64,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::ResetInstance { user_address, instance_id };
    let execute_info = cosmwasm_std::testing::message_info(&admin, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

#[test]
fn test_reset_instance_success_recurrent() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow).unwrap();
    
    // Create a recurrent instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance).unwrap();
    
    // Verify instance is created and last_executed_action is None
    let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Running);
    assert_eq!(instance_query.instance.last_executed_action, None);
    assert_eq!(instance_query.instance.base.execution_type, ExecutionType::Recurrent);
    
    // Manually set last_executed_action to simulate an executed action
    use auto_workflow_manager::state::{load_workflow_instance, save_workflow_instance};
    let mut instance = load_workflow_instance(deps.as_ref().storage, &user1, &1).unwrap();
    instance.last_executed_action = Some("start_action".to_string());
    save_workflow_instance(deps.as_mut().storage, &user1, &1, &instance).unwrap();
    
    // Verify last_executed_action is now set
    let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.last_executed_action, Some("start_action".to_string()));
    
    // Reset the instance
    let response = reset_instance(&mut deps, env.clone(), admin_address.clone(), user1.to_string(), 1).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "reset_instance");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user1.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "execution_type");
    assert_eq!(response.attributes[3].value, "recurrent");
    
    // Verify instance last_executed_action is still None (reset successful)
    let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.last_executed_action, None);
}

#[test]
fn test_reset_instance_success_oneshot() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow).unwrap();
    
    // Create a oneshot instance
    let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance).unwrap();
    
    // Verify instance is created and is OneShot
    let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.base.execution_type, ExecutionType::OneShot);
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Running);
    
    // Reset the OneShot instance - should change state to Finished
    let response = reset_instance(&mut deps, env.clone(), admin_address.clone(), user1.to_string(), 1).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "reset_instance");
    assert_eq!(response.attributes[1].key, "user_address");
    assert_eq!(response.attributes[1].value, user1.to_string());
    assert_eq!(response.attributes[2].key, "instance_id");
    assert_eq!(response.attributes[2].value, "1");
    assert_eq!(response.attributes[3].key, "execution_type");
    assert_eq!(response.attributes[3].value, "oneshot");
    
    // Verify instance state is now Finished
    let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance_query.instance.state, WorkflowInstanceState::Finished);
}

#[test]
fn test_reset_instance_not_found() {
    let (mut deps, env, api, admin_address, _publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Try to reset a non-existent instance - should fail
    let result = reset_instance(&mut deps, env.clone(), admin_address.clone(), user1.to_string(), 999);
    assert!(result.is_err());
    
    // Verify the error message
    match result.unwrap_err() {
        auto_workflow_manager::ContractError::InstanceNotFound { instance_id } => {
            assert_eq!(instance_id, "999");
        }
        _ => panic!("Expected InstanceNotFound"),
    }
}

#[test]
fn test_reset_instance_unauthorized() {
    let (mut deps, env, api, _admin_address, publisher_address, executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow).unwrap();
    
    // Create a recurrent instance
    let instance = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance).unwrap();
    
    // Try to reset the instance with non-admin address - should fail
    let result = reset_instance(&mut deps, env.clone(), executor_address.clone(), user1.to_string(), 1);
    assert!(result.is_err());
    
    // Verify the error message
    match result.unwrap_err() {
        auto_workflow_manager::ContractError::Unauthorized {} => {}
        _ => panic!("Expected Unauthorized"),
    }
}

#[test]
fn test_reset_instance_invalid_user_address() {
    let (mut deps, env, _api, admin_address, _publisher_address, _executor_address) = create_test_environment();
    
    // Try to reset with invalid user address - should fail
    let result = reset_instance(&mut deps, env.clone(), admin_address.clone(), "invalid_address".to_string(), 1);
    assert!(result.is_err());
    
    // Verify the error message
    match result.unwrap_err() {
        auto_workflow_manager::ContractError::Std(_) => {}
        _ => panic!("Expected Std error for invalid address"),
    }
}