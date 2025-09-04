use auto_workflow_manager::ContractError;
use cosmwasm_std::{Addr, MessageInfo, Timestamp};
use std::collections::{HashMap, HashSet};

mod utils;
use utils::{create_test_environment, publish_workflow};
use auto_workflow_manager::msg::{ActionMsg, ActionParamValue, NewWorkflowMsg, WorkflowVisibility, Template};

#[test]
fn test_publish_workflow_with_whitelist() {
    let (mut deps, env, api, _admin_address, publisher_address, _executor_address) = create_test_environment();

    // Create workflow with whitelisted contracts
    let mut whitelisted_contracts = HashSet::new();
    let token_address = api.addr_make("token_address");
    let staking_address = api.addr_make("staking_address");
    whitelisted_contracts.insert(token_address.to_string());
    whitelisted_contracts.insert(staking_address.to_string());

    let mut actions = HashMap::new();
    
    // Create action with template
    let mut templates = HashMap::new();
    templates.insert(
        "stake_template".to_string(),
        Template {
            contract: "{{token_address}}".to_string(),
            message: "{\"stake\": {\"amount\": \"{{amount}}\"}}".to_string(),
            funds: vec![],
        },
    );

    let mut params = HashMap::new();
    params.insert(
        "token_address".to_string(),
        ActionParamValue::String(token_address.to_string()),
    );
    params.insert(
        "amount".to_string(),
        ActionParamValue::String("1000000".to_string()),
    );

    actions.insert(
        "stake_tokens".to_string(),
        ActionMsg {
            params,
            next_actions: HashSet::new(),
            templates,
            whitelisted_contracts,
        },
    );

    let workflow_msg = NewWorkflowMsg {
        id: "test-whitelist-workflow".to_string(),
        start_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        visibility: WorkflowVisibility::Public,
        actions,
    };

    let response = publish_workflow(deps.as_mut(), env, publisher_address.clone(), workflow_msg).unwrap();

    // Verify response attributes
    assert_eq!(response.attributes.len(), 4);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "publish_workflow");
    assert_eq!(response.attributes[1].key, "workflow_id");
    assert_eq!(response.attributes[1].value, "test-whitelist-workflow");
    assert_eq!(response.attributes[2].key, "publisher");
    assert_eq!(response.attributes[2].value, publisher_address.to_string());
    assert_eq!(response.attributes[3].key, "state");
    assert_eq!(response.attributes[3].value, "approved");
}

#[test]
fn test_execute_action_with_whitelisted_contract() {
    let (mut deps, env, api, _admin_address, publisher_address, executor_address) = create_test_environment();

    // Create workflow with whitelisted contracts
    let mut whitelisted_contracts = HashSet::new();
    let contract_to_call = api.addr_make("contract_to_call");
    whitelisted_contracts.insert(contract_to_call.to_string());

    let mut actions = HashMap::new();
    
    let mut templates = HashMap::new();
    templates.insert(
        "stake_template".to_string(),
        Template {
            contract: "{{token_address}}".to_string(),
            message: "{\"stake\": {\"amount\": \"{{amount}}\"}}".to_string(),
            funds: vec![],
        },
    );

    let mut params = HashMap::new();
    params.insert(
        "token_address".to_string(),
        ActionParamValue::String(contract_to_call.to_string()),
    );
    params.insert(
        "amount".to_string(),
        ActionParamValue::String("1000000".to_string()),
    );

    actions.insert(
        "stake_tokens".to_string(),
        ActionMsg {
            params,
            next_actions: HashSet::new(),
            templates,
            whitelisted_contracts,
        },
    );

    let workflow_msg = NewWorkflowMsg {
        id: "test-whitelist-workflow".to_string(),
        start_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        visibility: WorkflowVisibility::Public,
        actions,
    };

    // Publish workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Create instance
    let user_address = api.addr_make("user");
    let instance_msg = auto_workflow_manager::msg::NewInstanceMsg {
        workflow_id: "test-whitelist-workflow".to_string(),
        onchain_parameters: HashMap::new(),
        offchain_parameters: HashMap::new(),
        execution_type: auto_workflow_manager::msg::ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(env.block.time.seconds() + 3600),
    };

    let response = auto_workflow_manager::execute::execute_instance(
        deps.as_mut(),
        env.clone(),
        MessageInfo {
            sender: Addr::unchecked(&user_address),
            funds: [].to_vec(),
        },
        instance_msg,
    ).unwrap();

    let instance_id = response.attributes[1].value.parse::<u64>().unwrap();

    // Execute action with whitelisted contract
    let mut action_params = HashMap::new();
    action_params.insert(
        "amount".to_string(),
        ActionParamValue::String("2000000".to_string()),
    );

    let result = auto_workflow_manager::execute::execute_action(
        deps.as_mut(),
        env,
        MessageInfo {
            sender: Addr::unchecked(&executor_address),
            funds: [].to_vec(),
        },
        user_address.to_string(),
        instance_id,
        "stake_tokens".to_string(),
        "stake_template".to_string(),
        Some(action_params),
    );

    // Should succeed because the resolved contract is in the whitelist
    assert!(result.is_ok());
}

#[test]
fn test_execute_action_with_non_whitelisted_contract() {
    let (mut deps, env, api, _admin_address, publisher_address, executor_address) = create_test_environment();

    // Create workflow with whitelisted contracts
    let mut whitelisted_contracts = HashSet::new();
    whitelisted_contracts.insert("osmo1token123456789abcdefghijklmnopqrstuvwxyz".to_string());

    let mut actions = HashMap::new();
    
    let mut templates = HashMap::new();
    templates.insert(
        "stake_template".to_string(),
        Template {
            contract: "{{token_address}}".to_string(),
            message: "{\"stake\": {\"amount\": \"{{amount}}\"}}".to_string(),
            funds: vec![],
        },
    );

    let mut params = HashMap::new();
    params.insert(
        "token_address".to_string(),
        ActionParamValue::String("osmo1malicious123456789abcdefghijklmnopqrstuvwxyz".to_string()),
    );
    params.insert(
        "amount".to_string(),
        ActionParamValue::String("1000000".to_string()),
    );

    actions.insert(
        "stake_tokens".to_string(),
        ActionMsg {
            params,
            next_actions: HashSet::new(),
            templates,
            whitelisted_contracts,
        },
    );

    let workflow_msg = NewWorkflowMsg {
        id: "test-whitelist-workflow".to_string(),
        start_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        visibility: WorkflowVisibility::Public,
        actions,
    };

    // Publish workflow
    publish_workflow(deps.as_mut(), env.clone(), publisher_address.clone(), workflow_msg).unwrap();

    // Create instance
    let user_address = api.addr_make("user");
    let instance_msg = auto_workflow_manager::msg::NewInstanceMsg {
        workflow_id: "test-whitelist-workflow".to_string(),
        onchain_parameters: HashMap::new(),
        offchain_parameters: HashMap::new(),
        execution_type: auto_workflow_manager::msg::ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(env.block.time.seconds() + 3600),
    };

    let response = auto_workflow_manager::execute::execute_instance(
        deps.as_mut(),
        env.clone(),
        MessageInfo {
            sender: Addr::unchecked(&user_address),
            funds: [].to_vec(),
        },
        instance_msg,
    ).unwrap();

    let instance_id = response.attributes[1].value.parse::<u64>().unwrap();

    // Execute action with non-whitelisted contract
    let mut action_params = HashMap::new();
    action_params.insert(
        "amount".to_string(),
        ActionParamValue::String("2000000".to_string()),
    );

    let result = auto_workflow_manager::execute::execute_action(
        deps.as_mut(),
        env,
        MessageInfo {
            sender: Addr::unchecked(&executor_address),
            funds: [].to_vec(),
        },
        user_address.to_string(),
        instance_id,
        "stake_tokens".to_string(),
        "stake_template".to_string(),
        Some(action_params),
    );

    // Should fail because the resolved contract is not in the whitelist
    assert!(result.is_err());
    
    match result {
        Err(ContractError::ContractNotWhitelisted { contract, workflow_id }) => {
            assert_eq!(contract, "osmo1malicious123456789abcdefghijklmnopqrstuvwxyz");
            assert_eq!(workflow_id, "test-whitelist-workflow");
        }
        _ => panic!("Expected ContractNotWhitelisted error, got different error"),
    }
} 