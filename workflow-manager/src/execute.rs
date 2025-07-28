use std::collections::HashMap;

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, SubMsg};

use crate::{
    msg::{NewInstanceMsg, ParamId},
    state::{
        load_next_instance_id, load_workflow, load_workflow_action, load_workflow_action_params, load_workflow_instance, load_workflow_instance_params, remove_workflow_instance, save_workflow, save_workflow_action, save_workflow_action_params, save_workflow_instance, save_workflow_instance_params, validate_sender_is_action_executor, validate_sender_is_publisher, Action, Workflow, WorkflowInstance
    },
};
use crate::{msg::{ActionParamValue, ActionType, ExecutionType, NewWorkflowMsg, WorkflowInstanceState, WorkflowState, WorkflowVisibility}, ContractError};

pub fn publish_workflow(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    input_workflow: NewWorkflowMsg,
) -> Result<Response, ContractError> {
    validate_sender_is_publisher(deps.storage, &info)?;

    // Check if workflow already exists
    if load_workflow(deps.storage, &input_workflow.id).is_ok() {
        return Err(ContractError::WorkflowAlreadyExists {
            workflow_id: input_workflow.id.clone(),
        });
    }

    let new_workflow = Workflow {
        start_action: input_workflow.start_action,
        visibility: input_workflow.visibility,
        publisher: info.sender.clone(),
        state: WorkflowState::Approved,
    };

    save_workflow(deps.storage, &input_workflow.id, &new_workflow)?;
    for (action_id, action) in input_workflow.actions {
        let new_action = Action {
            action_type: action.action_type,
            next_actions: action.next_actions,
            final_state: action.final_state,
        };
        save_workflow_action(deps.storage, &input_workflow.id, &action_id, &new_action)?;
        save_workflow_action_params(deps.storage, &input_workflow.id, &action_id, &action.params)?;
    }

    Ok(Response::new()
        .add_attribute("method", "publish_workflow")
        .add_attribute("workflow_id", input_workflow.id)
        .add_attribute("publisher", info.sender.to_string()))
}

pub fn execute_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance: NewInstanceMsg,
) -> Result<Response, ContractError> {
    // Check if workflow exists
    let workflow = load_workflow(deps.storage, &instance.workflow_id).map_err(|_| {
        ContractError::WorkflowNotFound {
            workflow_id: instance.workflow_id.clone(),
        }
    })?;

    // Check if workflow is approved
    if !matches!(workflow.state, WorkflowState::Approved) {
        return Err(ContractError::WorkflowNotApproved {
            workflow_id: instance.workflow_id.clone(),
        });
    }

    // Check if workflow is private and sender is not the publisher
    if matches!(workflow.visibility, WorkflowVisibility::Private)
        && info.sender != workflow.publisher
    {
        return Err(ContractError::PrivateWorkflowExecutionDenied {
            workflow_id: instance.workflow_id.clone(),
        });
    }

    // Generate auto-incremental ID for the instance
    let instance_id = load_next_instance_id(deps.storage)?;

    // Set initial state
    let new_instance: WorkflowInstance = WorkflowInstance {
        workflow_id: instance.workflow_id,
        state: WorkflowInstanceState::Running,
        last_executed_action: None,
        execution_type: instance.execution_type,
        expiration_time: instance.expiration_time,
    };

    // Save the instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &new_instance)?;
    save_workflow_instance_params(deps.storage, &info.sender, &instance_id, &instance.onchain_parameters)?;

    Ok(Response::new()
        .add_attribute("method", "execute_instance")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("workflow_id", new_instance.workflow_id)
        .add_attribute("requester", info.sender.to_string()))
}

pub fn cancel_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let _instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Remove the instance
    remove_workflow_instance(deps.storage, &info.sender, &instance_id)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_instance")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("canceller", info.sender.to_string()))
}

pub fn pause_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let mut instance = load_workflow_instance(deps.storage, &info.sender, &instance_id)
        .map_err(|_| ContractError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })?;

    // Check if instance is running
    if !matches!(instance.state, WorkflowInstanceState::Running) {
        return Err(ContractError::GenericError(
            "Instance is not running".to_string(),
        ));
    }

    // Pause the instance
    instance.state = WorkflowInstanceState::Paused;

    // Save the updated instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &instance)?;

    Ok(Response::new()
        .add_attribute("method", "pause_instance")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("pauser", info.sender.to_string()))
}

pub fn resume_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let mut instance = load_workflow_instance(deps.storage, &info.sender, &instance_id)
        .map_err(|_| ContractError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })?;

    // Check if instance is paused
    if !matches!(instance.state, WorkflowInstanceState::Paused) {
        return Err(ContractError::GenericError(
            "Instance is not paused".to_string(),
        ));
    }

    // Resume the instance
    instance.state = WorkflowInstanceState::Running;

    // Save the updated instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &instance)?;

    Ok(Response::new()
        .add_attribute("method", "resume_instance")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("resumer", info.sender.to_string()))
}

pub fn execute_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    instance_id: u64,
    action_id: String,
    params: Option<HashMap<String, ActionParamValue>>,
) -> Result<Response, ContractError> {
    // Validate sender is action executor
    validate_sender_is_action_executor(deps.storage, &info)?;

    // Load user instance
    let user_addr = deps.api.addr_validate(&user_address)?;
    let user_instance: WorkflowInstance = load_workflow_instance(deps.storage, &user_addr, &instance_id)?;

    // Validate instance expiration time
    if env.block.time >= user_instance.expiration_time {
        return Err(ContractError::GenericError(
            "Instance has expired".to_string(),
        ));
    }

    // Load workflow from user_instance.workflow_id
    let workflow = load_workflow(deps.storage, &user_instance.workflow_id)?;

    // Get the action to execute using the action_id parameter
    let action_to_execute = load_workflow_action(deps.storage, &user_instance.workflow_id, &action_id)
        .map_err(|_| ContractError::ActionNotFound {
            workflow_id: user_instance.workflow_id.clone(),
            action_id: action_id.clone(),
        })?;

    let can_execute = match &user_instance.last_executed_action {
        None => action_id == workflow.start_action,
        Some(last_executed_action_id) => {
            let last_executed_action = load_workflow_action(deps.storage, &user_instance.workflow_id, &last_executed_action_id)
                .map_err(|_| ContractError::ActionNotFound {
                workflow_id: user_instance.workflow_id.clone(),
                action_id: last_executed_action_id.clone(),
            })?;
            last_executed_action.next_actions.contains(&action_id) 
            || (user_instance.execution_type == ExecutionType::Recurrent && last_executed_action.final_state && action_id == workflow.start_action)
        }
    };

    if !can_execute {
        return Err(ContractError::GenericError(
            "Action cannot be executed: not first execution, not valid next action, and not recurrent start action".to_string()
        ));
    }

    // Get action parameters and create new HashMap
    let action_params = load_workflow_action_params(deps.storage, &user_instance.workflow_id, &action_id)?;
    let instance_params = load_workflow_instance_params(deps.storage, &user_addr, &instance_id)?;
    let mut resolved_params = HashMap::<String, ActionParamValue>::new();

    for (key, value) in action_params {
        // si param.value es #ip.requester => busco user_instance.requester
        // si param.value comienza con #ip, busco en user_instance.params
        // si param.value comienza con #cp, busco en execute_action_params
        // else es un valor fijo
        let resolved_value = resolve_param_value(&value, &user_addr,&instance_params, &params)?;
        resolved_params.insert(key.clone(), resolved_value);
    }

    // Create sub messages based on action type
    let sub_msgs: Vec<SubMsg> = match action_to_execute.action_type {
        ActionType::StakedTokenClaimer => staked_token_claimer(resolved_params)?,
        ActionType::TokenStaker => token_staker(resolved_params)?,
    };

    // Update instance with last executed action
    let mut updated_instance = user_instance;
    updated_instance.last_executed_action = Some(action_id.clone());
    save_workflow_instance(deps.storage, &user_addr, &instance_id, &updated_instance)?;

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attribute("method", "execute_action")
        .add_attribute("user_address", user_address)
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("action_id", action_id))
}

fn resolve_param_value(
    param_value: &ActionParamValue,
    user_addr: &Addr,
    instance_params: &HashMap<ParamId, ActionParamValue>,
    execute_action_params: &Option<HashMap<ParamId, ActionParamValue>>,
) -> Result<ActionParamValue, ContractError> {
    let value_str = match param_value {
        ActionParamValue::String(s) => s,
        ActionParamValue::BigInt(s) => s,
    };

    if value_str == "#ip.requester" {
        Ok(ActionParamValue::String(
            user_addr.to_string(),
        ))
    } else if value_str.starts_with("#ip.") {
        // Extract the key after #ip.
        let key = &value_str[4..];
        if let Some(value) = instance_params.get(key) {
            Ok(value.clone())
        } else {
            Err(ContractError::GenericError(format!(
                "Parameter '{}' not found in instance parameters",
                key
            )))
        }
    } else if value_str.starts_with("#cp.") {
        // Extract the key after #cp.
        let key = &value_str[4..];
        if let Some(params) = execute_action_params {
            if let Some(value) = params.get(key) {
                Ok(value.clone())
            } else {
                Err(ContractError::GenericError(format!(
                    "Parameter '{}' not found in execute action parameters",
                    key
                )))
            }
        } else {
            Err(ContractError::GenericError(
                "Execute action parameters not provided".to_string(),
            ))
        }
    } else {
        Ok(param_value.clone()) // Fixed value
    }
}

// Placeholder functions for action types
fn staked_token_claimer(
    _params: HashMap<String, ActionParamValue>,
) -> Result<Vec<SubMsg>, ContractError> {
    // TODO: Implement staked token claimer logic
    Ok(vec![])
}

fn token_staker(_params: HashMap<String, ActionParamValue>) -> Result<Vec<SubMsg>, ContractError> {
    // TODO: Implement token staker logic
    Ok(vec![])
}
