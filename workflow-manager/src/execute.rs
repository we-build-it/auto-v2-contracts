use std::collections::HashMap;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, SubMsg};

use crate::{
    msg::NewInstanceMsg,
    state::{
        get_next_instance_id, load_workflow, load_workflow_instance, remove_workflow_instance,
        save_workflow, save_workflow_instance, validate_sender_is_action_executor,
        validate_sender_is_publisher, ActionParamValue, ActionType, ExecutionType, Workflow,
        WorkflowInstance, WorkflowInstanceState, WorkflowState, WorkflowVisibility,
    },
};
use crate::{msg::NewWorkflowMsg, ContractError};

pub fn publish_workflow(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    input_workflow: NewWorkflowMsg,
) -> Result<Response, ContractError> {
    validate_sender_is_publisher(deps.storage, &info)?;

    // Check if workflow already exists
    if load_workflow(deps.storage, &input_workflow.id).is_ok() {
        return Err(ContractError::TemplateAlreadyExists {
            template_id: input_workflow.id.clone(),
        });
    }

    let new_workflow = Workflow {
        id: input_workflow.id.clone(),
        start_action: input_workflow.start_action,
        visibility: input_workflow.visibility,
        actions: input_workflow.actions,
        publisher: info.sender.clone(),
        state: WorkflowState::Approved,
    };

    save_workflow(deps.storage, &new_workflow)?;

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
            template_id: instance.workflow_id.clone(),
        }
    })?;

    // Check if workflow is approved
    if !matches!(workflow.state, WorkflowState::Approved) {
        return Err(ContractError::WorkflowNotApproved {
            template_id: instance.workflow_id.clone(),
        });
    }

    // Check if workflow is private and sender is not the publisher
    if matches!(workflow.visibility, WorkflowVisibility::Private)
        && info.sender != workflow.publisher
    {
        return Err(ContractError::PrivateWorkflowExecutionDenied {
            template_id: instance.workflow_id.clone(),
        });
    }

    // Generate auto-incremental ID for the instance
    let instance_id = get_next_instance_id(deps.storage)?;

    // Set initial state
    let new_instance: WorkflowInstance = WorkflowInstance {
        id: instance_id,
        workflow_id: instance.workflow_id,
        state: WorkflowInstanceState::Running,
        requester: info.sender.clone(),
        last_executed_action: None,
        onchain_parameters: instance.onchain_parameters,
        execution_type: instance.execution_type,
        expiration_time: instance.expiration_time,
    };

    // Save the instance
    save_workflow_instance(deps.storage, info.sender.clone(), &new_instance)?;

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
    let instance =
        load_workflow_instance(deps.storage, info.sender.clone(), instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                flow_id: instance_id.to_string(),
            }
        })?;

    // Validate that the requester is the sender
    if instance.requester != info.sender {
        return Err(ContractError::InstanceAccessUnauthorized {
            action: "cancel".to_string(),
            instance_id: instance_id.to_string(),
        });
    }

    // Remove the instance
    remove_workflow_instance(deps.storage, info.sender.clone(), instance_id)?;

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
    let mut instance = load_workflow_instance(deps.storage, info.sender.clone(), instance_id)
        .map_err(|_| ContractError::InstanceNotFound {
            flow_id: instance_id.to_string(),
        })?;

    // Validate that the requester is the sender
    if instance.requester != info.sender {
        return Err(ContractError::InstanceAccessUnauthorized {
            action: "pause".to_string(),
            instance_id: instance_id.to_string(),
        });
    }

    // Check if instance is running
    if !matches!(instance.state, WorkflowInstanceState::Running) {
        return Err(ContractError::GenericError(
            "Instance is not running".to_string(),
        ));
    }

    // Pause the instance
    instance.state = WorkflowInstanceState::Paused;

    // Save the updated instance
    save_workflow_instance(deps.storage, info.sender.clone(), &instance)?;

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
    let mut instance = load_workflow_instance(deps.storage, info.sender.clone(), instance_id)
        .map_err(|_| ContractError::InstanceNotFound {
            flow_id: instance_id.to_string(),
        })?;

    // Validate that the requester is the sender
    if instance.requester != info.sender {
        return Err(ContractError::InstanceAccessUnauthorized {
            action: "resume".to_string(),
            instance_id: instance_id.to_string(),
        });
    }

    // Check if instance is paused
    if !matches!(instance.state, WorkflowInstanceState::Paused) {
        return Err(ContractError::GenericError(
            "Instance is not paused".to_string(),
        ));
    }

    // Resume the instance
    instance.state = WorkflowInstanceState::Running;

    // Save the updated instance
    save_workflow_instance(deps.storage, info.sender.clone(), &instance)?;

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
    params: Option<HashMap<String, String>>,
) -> Result<Response, ContractError> {
    // Validate sender is action executor
    validate_sender_is_action_executor(deps.storage, &info)?;

    // Load user instance
    let user_addr = deps.api.addr_validate(&user_address)?;
    let user_instance = load_workflow_instance(deps.storage, user_addr.clone(), instance_id)?;

    // Validate instance expiration time
    if env.block.time >= user_instance.expiration_time {
        return Err(ContractError::GenericError(
            "Instance has expired".to_string(),
        ));
    }

    // Load workflow from user_instance.workflow_id
    let workflow = load_workflow(deps.storage, &user_instance.workflow_id)?;

    // Get last executed action as Option<&Action> and action to execute by action_id
    let last_executed_action = if let Some(last_action_id) = &user_instance.last_executed_action {
        workflow.actions.get(last_action_id)
    } else {
        None
    };

    // Get the action to execute using the action_id parameter
    let action_to_execute =
        workflow
            .actions
            .get(&action_id)
            .ok_or_else(|| ContractError::ActionNotFound {
                template_id: user_instance.workflow_id.clone(),
                action_id: action_id.clone(),
            })?;

    // Check if the action can be executed based on workflow rules
    let is_first_execution =
        user_instance.last_executed_action.is_none() && action_id == workflow.start_action;

    let is_valid_next_action = user_instance.last_executed_action.is_some()
        && last_executed_action
            .unwrap()
            .next_actions
            .contains(&action_id);

    let is_recurrent_start_action = user_instance.execution_type == ExecutionType::Recurrent
        && action_id == workflow.start_action
        && (user_instance.last_executed_action.is_none()
            || last_executed_action.unwrap().final_state);

    let can_execute = is_first_execution || is_valid_next_action || is_recurrent_start_action;

    if !can_execute {
        return Err(ContractError::GenericError(
            "Action cannot be executed: not first execution, not valid next action, and not recurrent start action".to_string()
        ));
    }

    // Get action parameters and create new HashMap
    let action_params = &action_to_execute.params;
    let mut resolved_params = HashMap::<String, ActionParamValue>::new();

    for (key, value) in action_params {
        // si param.value es #ip.requester => busco user_instance.requester
        // si param.value comienza con #ip, busco en user_instance.params
        // si param.value comienza con #cp, busco en execute_action_params
        // else es un valor fijo
        let resolved_value = resolve_param_value(&value, &user_instance, &params)?;
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
    save_workflow_instance(deps.storage, user_addr.clone(), &updated_instance)?;

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attribute("method", "execute_action")
        .add_attribute("user_address", user_address)
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("action_id", action_id))
}

fn resolve_param_value(
    param_value: &ActionParamValue,
    user_instance: &WorkflowInstance,
    execute_action_params: &Option<HashMap<String, String>>,
) -> Result<ActionParamValue, ContractError> {
    let value_str = match param_value {
        ActionParamValue::String(s) => s,
        ActionParamValue::BigInt(s) => s,
    };

    if value_str == "#ip.requester" {
        Ok(ActionParamValue::String(
            user_instance.requester.to_string(),
        ))
    } else if value_str.starts_with("#ip.") {
        // Extract the key after #ip.
        let key = &value_str[4..];
        if let Some(value) = user_instance.onchain_parameters.get(key) {
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
                Ok(ActionParamValue::String(value.clone()))
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
