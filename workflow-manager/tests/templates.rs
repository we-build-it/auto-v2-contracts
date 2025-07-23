use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
use workflow_manager::ContractError;
use std::collections::HashSet;

mod utils;
use utils::{create_simple_test_workflow, create_test_workflow, instantiate_contract, publish_workflow};

#[test]
fn test_publish_template_ok() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let publisher_address = api.addr_make("publisher");
    
    let allowed_publishers = HashSet::from([publisher_address.clone()]);
    let allowed_action_executors = HashSet::from([api.addr_make("executor")]);
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        allowed_publishers,
        allowed_action_executors,
    ).unwrap();

    // Create and publish workflow using utils functions
    let workflow_msg = create_test_workflow();
    let response =
        publish_workflow(deps.as_mut(), env, publisher_address.clone(), workflow_msg).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 3);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "publish_workflow");
    assert_eq!(response.attributes[1].key, "workflow_id");
    assert_eq!(response.attributes[1].value, "test-workflow");
    assert_eq!(response.attributes[2].key, "publisher");
    assert_eq!(response.attributes[2].value, publisher_address.to_string());
}

#[test]
fn test_publish_template_invalid_publisher() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let unauthorized_publisher = api.addr_make("unauthorized_publisher");
    
    let allowed_publishers = HashSet::from([api.addr_make("allowed_publisher")]);
    let allowed_action_executors = HashSet::from([api.addr_make("executor")]);
    
    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address,
        allowed_publishers,
        allowed_action_executors,
    ).unwrap();

    // Try to publish workflow with unauthorized publisher using utils functions
    let workflow_msg = create_simple_test_workflow();
    let result = publish_workflow(
        deps.as_mut(),
        env,
        unauthorized_publisher.clone(),
        workflow_msg,
    );

    // Verify that the operation fails
    assert!(result.is_err());
    
    // Check that the error is the expected one
    match result {
        Err(ContractError::Unauthorized { .. }) => {
            // Expected error
        }
        _ => panic!("Expected Unauthorized error, got different error"),
    }
}
