use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Addr, DepsMut, Empty, Env, OwnedDeps, Response
};
use std::collections::{HashMap, HashSet};

use auto_workflow_manager::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg, NewWorkflowMsg, WorkflowVisibility, ActionMsg, ActionParamValue, Template},
};

/// Initialize the contract with the given parameters
pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    admin: Addr,
    allowed_publishers: HashSet<Addr>,
    allowed_action_executors: HashSet<Addr>,
) -> Result<Response, auto_workflow_manager::error::ContractError> {
    let instantiate_msg = InstantiateMsg {
        allowed_publishers,
        allowed_action_executors,
        referral_memo: "test-referral-memo".to_string(),
    };
    
    let instantiate_info = message_info(&admin, &[]);
    instantiate(deps, env, instantiate_info, instantiate_msg)
}

#[allow(dead_code)]
pub fn create_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "test-workflow".to_string(),
        start_action: "stake_tokens".to_string(),
        visibility: WorkflowVisibility::Public,

        actions: HashMap::from([
            (
                "stake_tokens".to_string(),
                ActionMsg {
                    params: HashMap::from([
                        (
                            "amount".to_string(),
                            ActionParamValue::String("1000000".to_string()),
                        ),
                        (
                            "token_address".to_string(),
                            ActionParamValue::String(
                                "osmo1token123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                            ),
                        ),
                    ]),
                    next_actions: HashSet::from(["claim_rewards".to_string()]),
                    final_state: false,
                    templates: HashMap::from([
                        (
                            "default".to_string(),
                            Template {
                                contract: "{{token_address}}".to_string(),
                                message: "{\"stake\":{ \"amount\": {{amount}} }}".to_string(),
                                funds: vec![],
                            },
                        ),
                    ]),
                    whitelisted_contracts: HashSet::from([
                        "osmo1token123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                    ]),
                },
            ),
            (
                "claim_rewards".to_string(),
                ActionMsg {
                    params: HashMap::from([(
                        "staking_contract".to_string(),
                        ActionParamValue::String(
                            "osmo1staking123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                        ),
                    )]),
                    next_actions: HashSet::new(),
                    final_state: true,
                    templates: HashMap::from([
                        (
                            "default".to_string(),
                            Template {
                                contract: "{{staking_contract}}".to_string(),
                                message: "{\"claim\":{}}".to_string(),
                                funds: vec![],
                            },
                        ),
                    ]),
                    whitelisted_contracts: HashSet::from([
                        "osmo1staking123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                    ]),
                },
            ),
        ]),
    }
}

#[allow(dead_code)]
pub fn create_simple_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "simple-test-workflow".to_string(),
        start_action: "stake_tokens".to_string(),
        visibility: WorkflowVisibility::Public,

        actions: HashMap::from([(
            "stake_tokens".to_string(),
            ActionMsg {
                params: HashMap::from([(
                    "amount".to_string(),
                    ActionParamValue::String("1000000".to_string()),
                )]),
                next_actions: HashSet::new(),
                final_state: true,
                templates: HashMap::from([
                    (
                        "default".to_string(),
                        Template {
                            contract: "osmo1token123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                            message: "{\"stake\":{ \"amount\": {{amount}} }}".to_string(),
                            funds: vec![],
                        },
                    ),
                ]),
                whitelisted_contracts: HashSet::from([
                    "osmo1token123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                ]),
            },
        )]),
    }
}

#[allow(dead_code)]
pub fn create_template_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "template-test-workflow".to_string(),
        start_action: "claim_tokens".to_string(),
        visibility: WorkflowVisibility::Public,

        actions: HashMap::from([
            (
                "claim_tokens".to_string(),
                ActionMsg {
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
                    final_state: true,
                    templates: HashMap::from([
                        (
                            "daodao".to_string(),
                            Template {
                                contract: "{{contractAddress}}".to_string(),
                                message: "{\"claim\":{ \"id\": {{distributionId}} }}".to_string(),
                                funds: vec![],
                            },
                        ),
                        (
                            "rujira".to_string(),
                            Template {
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
    }
}

#[allow(dead_code)]
pub fn publish_workflow(
    deps: DepsMut,
    env: Env,
    publisher: Addr,
    workflow_msg: NewWorkflowMsg,
) -> Result<Response, auto_workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::PublishWorkflow {
        workflow: workflow_msg,
    };

    let execute_info = message_info(&publisher, &[]);
    execute(deps, env, execute_info, execute_msg)
}

#[allow(dead_code)]
pub fn create_test_environment() -> (OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>, Env, MockApi, Addr, Addr, Addr) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let api = deps.api;

    // Setup contract with parameters
    let admin_address = api.addr_make("admin");
    let publisher_address = api.addr_make("publisher");
    let executor_address = api.addr_make("executor");
    
    let allowed_publishers = HashSet::from([publisher_address.clone()]);
    let allowed_action_executors = HashSet::from([executor_address.clone()]);

    // Initialize contract using utils function
    instantiate_contract(
        deps.as_mut(),
        env.clone(),
        admin_address.clone(),
        allowed_publishers,
        allowed_action_executors,
    ).unwrap();

    (deps, env, api, admin_address.clone(), publisher_address.clone(), executor_address.clone())
}
