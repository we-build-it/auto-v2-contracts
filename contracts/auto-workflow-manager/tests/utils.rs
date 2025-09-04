use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Addr, DepsMut, Empty, Env, OwnedDeps, Response, Timestamp
};
use std::collections::{HashMap, HashSet};

use auto_workflow_manager::{
    contract::{execute, instantiate},
    msg::{ActionMsg, ActionParamValue, ExecuteMsg, ExecutionType, InstantiateMsg, NewInstanceMsg, NewWorkflowMsg, Template, WorkflowVisibility},
};

/// Initialize the contract with the given parameters
pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    admin: Addr,
    allowed_publishers: HashSet<Addr>,
    allowed_action_executors: HashSet<Addr>,
    fee_manager_address: Addr,
) -> Result<Response, auto_workflow_manager::error::ContractError> {
    let instantiate_msg = InstantiateMsg {
        allowed_publishers,
        allowed_action_executors,
        referral_memo: "test-referral-memo".to_string(),
        fee_manager_address: fee_manager_address,
    };
    
    let instantiate_info = message_info(&admin, &[]);
    instantiate(deps, env, instantiate_info, instantiate_msg)
}

#[allow(dead_code)]
pub fn create_test_workflow(api: MockApi) -> NewWorkflowMsg {
    let token_address = api.addr_make("token_address");
    let staking_address = api.addr_make("staking_address");
    NewWorkflowMsg {
        id: "test-workflow".to_string(),
        start_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "claim_rewards".to_string(),
        ]),
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
                                token_address.to_string(),
                            ),
                        ),
                    ]),
                    next_actions: HashSet::from(["claim_rewards".to_string()]),
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
                        token_address.to_string(),
                    ]),
                },
            ),
            (
                "claim_rewards".to_string(),
                ActionMsg {
                    params: HashMap::from([(
                        "staking_contract".to_string(),
                        ActionParamValue::String(
                            staking_address.to_string(),
                        ),
                    )]),
                    next_actions: HashSet::new(),
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
                        staking_address.to_string(),
                    ]),
                },
            ),
        ]),
    }
}

#[allow(dead_code)]
pub fn create_simple_test_workflow(api: MockApi) -> NewWorkflowMsg {
    let contract_address =  api.addr_make("contract_to_call");
    NewWorkflowMsg {
        id: "simple-test-workflow".to_string(),
        start_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "stake_tokens".to_string(),
        ]),
        visibility: WorkflowVisibility::Public,
        actions: HashMap::from([(
            "stake_tokens".to_string(),
            ActionMsg {
                params: HashMap::from([(
                    "amount".to_string(),
                    ActionParamValue::String("1000000".to_string()),
                )]),
                next_actions: HashSet::new(),
                templates: HashMap::from([
                    (
                        "default".to_string(),
                        Template {
                            contract: contract_address.to_string(),
                            message: "{\"stake\":{ \"amount\": {{amount}} }}".to_string(),
                            funds: vec![],
                        },
                    ),
                ]),
                whitelisted_contracts: HashSet::from([
                    contract_address.to_string(),
                ]),
            },
        )]),
    }
}

#[allow(dead_code)]
pub fn create_template_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "template-test-workflow".to_string(),
        start_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
        end_actions: HashSet::from([
            "claim_tokens".to_string(),
        ]),
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
pub fn create_oneshot_test_instance(workflow_id: String) -> NewInstanceMsg {
    NewInstanceMsg {
        workflow_id,
        onchain_parameters: std::collections::HashMap::new(),
        offchain_parameters: std::collections::HashMap::new(),
        execution_type: ExecutionType::OneShot,
        expiration_time: Timestamp::from_seconds(1000000000), // Far future
    }
}

#[allow(dead_code)]
pub fn execute_instance(
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
        api.addr_make("fee_manager_address"),
    ).unwrap();

    (deps, env, api, admin_address.clone(), publisher_address.clone(), executor_address.clone())
}
