use auto_workflow_manager::ContractError;

mod utils;
use utils::{create_simple_test_workflow, create_test_workflow, publish_workflow};

use crate::utils::create_test_environment;

#[test]
fn test_publish_workflow_ok() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();

    // Create and publish workflow using utils functions
    let workflow_msg = create_test_workflow(api);
    let response =
        publish_workflow(deps.as_mut(), env, publisher_address.clone(), workflow_msg).unwrap();

    // Verify response events and attributes
    assert_eq!(response.attributes.len(), 0);
    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "autorujira-workflow-manager/publish_workflow");
    assert_eq!(response.events[0].attributes.len(), 3);
    assert_eq!(response.events[0].attributes[0].key, "workflow_id");
    assert_eq!(response.events[0].attributes[0].value, "test-workflow");
    assert_eq!(response.events[0].attributes[1].key, "publisher");
    assert_eq!(response.events[0].attributes[1].value, publisher_address.to_string());
    assert_eq!(response.events[0].attributes[2].key, "state");
    assert_eq!(response.events[0].attributes[2].value, "approved");
}

#[test]
fn test_publish_workflow_invalid_publisher() {
    let (mut deps, env, api, _admin_address, _publisher_address, _executor_address) = create_test_environment();

    let unauthorized_publisher = api.addr_make("unauthorized_publisher");
    
    // Try to publish workflow with unauthorized publisher using utils functions
    let workflow_msg = create_simple_test_workflow(api);
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

