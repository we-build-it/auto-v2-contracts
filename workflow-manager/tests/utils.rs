use cosmwasm_std::{
    testing::{message_info},
    Addr, DepsMut, Env, Response,
};
use std::collections::{HashMap, HashSet};

use workflow_manager::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg, NewWorkflowMsg, ActionType, WorkflowVisibility, ActionMsg, ActionParamValue},
};

/// Initialize the contract with the given parameters
pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    admin: Addr,
    allowed_publishers: HashSet<Addr>,
    allowed_action_executors: HashSet<Addr>,
) -> Result<Response, workflow_manager::error::ContractError> {
    let instantiate_msg = InstantiateMsg {
        allowed_publishers,
        allowed_action_executors,
        referral_memo: "test-referral-memo".to_string(),
    };
    
    let instantiate_info = message_info(&admin, &[]);
    instantiate(deps, env, instantiate_info, instantiate_msg)
}

pub fn create_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "test-workflow".to_string(),
        start_action: "stake_tokens".to_string(),
        visibility: WorkflowVisibility::Public,
        actions: HashMap::from([
            (
                "stake_tokens".to_string(),
                ActionMsg {
                    action_type: ActionType::TokenStaker,
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
                },
            ),
            (
                "claim_rewards".to_string(),
                ActionMsg {
                    action_type: ActionType::StakedTokenClaimer,
                    params: HashMap::from([(
                        "staking_contract".to_string(),
                        ActionParamValue::String(
                            "osmo1staking123456789abcdefghijklmnopqrstuvwxyz".to_string(),
                        ),
                    )]),
                    next_actions: HashSet::new(),
                    final_state: true,
                },
            ),
        ]),
    }
}

pub fn create_simple_test_workflow() -> NewWorkflowMsg {
    NewWorkflowMsg {
        id: "simple-test-workflow".to_string(),
        start_action: "stake_tokens".to_string(),
        visibility: WorkflowVisibility::Public,
        actions: HashMap::from([(
            "stake_tokens".to_string(),
            ActionMsg {
                action_type: ActionType::TokenStaker,
                params: HashMap::from([(
                    "amount".to_string(),
                    ActionParamValue::String("1000000".to_string()),
                )]),
                next_actions: HashSet::new(),
                final_state: true,
            },
        )]),
    }
}

pub fn publish_workflow(
    deps: DepsMut,
    env: Env,
    publisher: Addr,
    workflow_msg: NewWorkflowMsg,
) -> Result<Response, workflow_manager::error::ContractError> {
    let execute_msg = ExecuteMsg::PublishWorkflow {
        workflow: workflow_msg,
    };

    let execute_info = message_info(&publisher, &[]);
    execute(deps, env, execute_info, execute_msg)
}
