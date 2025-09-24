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

fn finish_instances(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::testing::MockStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
        cosmwasm_std::Empty,
    >,
    env: cosmwasm_std::Env,
    admin: Addr,
    instances: Vec<auto_workflow_manager::msg::FinishInstanceRequest>,
) -> Result<cosmwasm_std::Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::FinishInstances { instances };
    let execute_info = cosmwasm_std::testing::message_info(&admin, &[]);
    execute(deps.as_mut(), env, execute_info, execute_msg)
}

#[test]
fn test_finish_instances_success() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    let user2 = api.addr_make("user2");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Create instances for user1
    let instance1 = create_oneshot_test_instance("simple-test-workflow".to_string());
    let instance2 = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance1).unwrap();
    execute_instance(&mut deps, env.clone(), user1.clone(), instance2).unwrap();
    
    // Create instances for user2
    let instance3 = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user2.clone(), instance3).unwrap();
    
    // Verify instances are in Running state
    let instance1_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance1_query.instance.state, WorkflowInstanceState::Running);
    
    let instance2_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 2).unwrap();
    assert_eq!(instance2_query.instance.state, WorkflowInstanceState::Running);
    
    let instance3_query = query_workflow_instance(deps.as_ref(), user2.to_string(), 3).unwrap();
    assert_eq!(instance3_query.instance.state, WorkflowInstanceState::Running);
    
    // Finish instances
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1, 2],
        },
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user2.to_string(),
            instance_ids: vec![3],
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "finish_instances");
    assert_eq!(response.attributes[1].key, "finished_instance_ids");
    // Check that the finished instances contain the expected IDs
    let finished_ids = response.attributes[1].value.clone();
    assert!(finished_ids.contains("1"));
    assert!(finished_ids.contains("2"));
    assert!(finished_ids.contains("3"));
    assert_eq!(response.attributes[2].key, "not_found_instance_ids");
    assert_eq!(response.attributes[2].value, "");
    assert_eq!(response.attributes[3].key, "already_finished_instance_ids");
    assert_eq!(response.attributes[3].value, "");
    
    // Verify instances are now Finished
    let instance1_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 1).unwrap();
    assert_eq!(instance1_query.instance.state, WorkflowInstanceState::Finished);
    
    let instance2_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 2).unwrap();
    assert_eq!(instance2_query.instance.state, WorkflowInstanceState::Finished);
    
    let instance3_query = query_workflow_instance(deps.as_ref(), user2.to_string(), 3).unwrap();
    assert_eq!(instance3_query.instance.state, WorkflowInstanceState::Finished);
}

#[test]
fn test_finish_instances_not_found() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Try to finish non-existent instances
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![999, 1000], // Non-existent instances
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "finish_instances");
    assert_eq!(response.attributes[1].key, "finished_instance_ids");
    assert_eq!(response.attributes[1].value, "");
    assert_eq!(response.attributes[2].key, "not_found_instance_ids");
    // Check that the not found instances contain the expected IDs
    let not_found_ids = response.attributes[2].value.clone();
    assert!(not_found_ids.contains("999"));
    assert!(not_found_ids.contains("1000"));
    assert_eq!(response.attributes[3].key, "already_finished_instance_ids");
    assert_eq!(response.attributes[3].value, "");
}

#[test]
fn test_finish_instances_already_finished() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Create instance
    let instance1 = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance1).unwrap();
    
    // Finish instance first time
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1],
        },
    ];
    finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Try to finish the same instance again
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1],
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "finish_instances");
    assert_eq!(response.attributes[1].key, "finished_instance_ids");
    assert_eq!(response.attributes[1].value, "");
    assert_eq!(response.attributes[2].key, "not_found_instance_ids");
    assert_eq!(response.attributes[2].value, "");
    assert_eq!(response.attributes[3].key, "already_finished_instance_ids");
    // Check that the already finished instances contain the expected ID
    let already_finished_ids = response.attributes[3].value.clone();
    assert!(already_finished_ids.contains("1"));
}

#[test]
fn test_finish_instances_mixed_scenarios() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    let user2 = api.addr_make("user2");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Create some instances
    let instance1 = create_oneshot_test_instance("simple-test-workflow".to_string());
    let instance2 = create_recurrent_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance1).unwrap();
    execute_instance(&mut deps, env.clone(), user1.clone(), instance2).unwrap();
    
    // Finish instance1 first time
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1],
        },
    ];
    finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Now try to finish: instance1 (already finished), instance2 (running), instance3 (non-existent)
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1, 2, 3], // 1=already finished, 2=running, 3=non-existent
        },
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user2.to_string(),
            instance_ids: vec![999], // Non-existent user and instance
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "finish_instances");
    assert_eq!(response.attributes[1].key, "finished_instance_ids");
    // Check that only instance2 was finished
    let finished_ids = response.attributes[1].value.clone();
    assert!(finished_ids.contains("2"));
    assert_eq!(response.attributes[2].key, "not_found_instance_ids");
    // Check that instance3 and 999 were not found
    let not_found_ids = response.attributes[2].value.clone();
    assert!(not_found_ids.contains("3"));
    assert!(not_found_ids.contains("999"));
    assert_eq!(response.attributes[3].key, "already_finished_instance_ids");
    // Check that instance1 was already finished
    let already_finished_ids = response.attributes[3].value.clone();
    assert!(already_finished_ids.contains("1"));
    
    // Verify instance2 is now Finished
    let instance2_query = query_workflow_instance(deps.as_ref(), user1.to_string(), 2).unwrap();
    assert_eq!(instance2_query.instance.state, WorkflowInstanceState::Finished);
}

#[test]
fn test_finish_instances_unauthorized() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    let unauthorized_user = api.addr_make("unauthorized");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Create instance
    let instance1 = create_oneshot_test_instance("simple-test-workflow".to_string());
    execute_instance(&mut deps, env.clone(), user1.clone(), instance1).unwrap();
    
    // Try to finish instances as unauthorized user
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: vec![1],
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), unauthorized_user, instances);
    
    // Should fail with unauthorized error
    assert!(response.is_err());
    match response.unwrap_err() {
        ContractError::Unauthorized { .. } => {
            // Expected error
        }
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_finish_instances_large_batch() {
    let (mut deps, env, api, admin_address, publisher_address, _executor_address) = create_test_environment();
    
    let user1 = api.addr_make("user1");
    
    // Initialize contract
    let workflow = create_simple_test_workflow(api);
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow.clone()).unwrap();
    
    // Create many instances (test with 10 instances)
    let mut instance_ids = Vec::new();
    for i in 1..=10 {
        let instance = create_oneshot_test_instance("simple-test-workflow".to_string());
        execute_instance(&mut deps, env.clone(), user1.clone(), instance).unwrap();
        instance_ids.push(i);
    }
    
    // Finish all instances at once
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: user1.to_string(),
            instance_ids: instance_ids,
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances).unwrap();
    
    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "finish_instances");
    assert_eq!(response.attributes[1].key, "finished_instance_ids");
    // Should contain all 10 instances
    let finished_ids = response.attributes[1].value.clone();
    for i in 1..=10 {
        assert!(finished_ids.contains(&i.to_string()));
    }
    assert_eq!(response.attributes[2].key, "not_found_instance_ids");
    assert_eq!(response.attributes[2].value, "");
    assert_eq!(response.attributes[3].key, "already_finished_instance_ids");
    assert_eq!(response.attributes[3].value, "");
    
    // Verify all instances are now Finished
    for i in 1..=10 {
        let instance_query = query_workflow_instance(deps.as_ref(), user1.to_string(), i).unwrap();
        assert_eq!(instance_query.instance.state, WorkflowInstanceState::Finished);
    }
}

#[test]
fn test_finish_instances_invalid_address() {
    let (mut deps, env, _api, admin_address, _publisher_address, _executor_address) = create_test_environment();
    
    // Try to finish instances with invalid address
    let instances = vec![
        auto_workflow_manager::msg::FinishInstanceRequest {
            requester: "invalid_address".to_string(),
            instance_ids: vec![1, 2],
        },
    ];
    
    let response = finish_instances(&mut deps, env.clone(), admin_address.clone(), instances);
    
    // Should fail with address validation error
    assert!(response.is_err());
    match response.unwrap_err() {
        ContractError::Std(cosmwasm_std::StdError::GenericErr { msg, .. }) => {
            assert!(msg.contains("Error decoding bech32"));
        }
        _ => panic!("Expected address validation error"),
    }
}