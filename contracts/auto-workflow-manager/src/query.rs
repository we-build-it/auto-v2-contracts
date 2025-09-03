use cosmwasm_std::{Addr, Deps, StdResult};
use crate::{
    msg::{ActionMsg, GetInstancesResponse, GetWorkflowInstanceResponse, GetWorkflowResponse, GetUserPaymentConfigResponse, InstanceId, NewInstanceMsg, NewWorkflowMsg, WorkflowInstanceResponse, WorkflowResponse}, 
    state::{load_workflow, load_workflow_action_params, load_workflow_action_templates, load_workflow_action_contracts, load_workflow_actions, load_workflow_instance, load_workflow_instance_params, load_workflow_instances_by_requester, load_user_payment_config, WorkflowInstance},
};

pub fn query_workflow_by_id(deps: Deps, workflow_id: String) -> StdResult<GetWorkflowResponse> {
    let workflow = load_workflow(deps.storage, &workflow_id)?;
    Ok(GetWorkflowResponse { workflow: WorkflowResponse {
        base: NewWorkflowMsg {
            id: workflow_id.clone(),
            start_actions: workflow.start_actions,
            end_actions: workflow.end_actions,
            visibility: workflow.visibility,
            actions: load_workflow_actions(deps.storage, &workflow_id)?.iter().map(|(action_id, action)| (action_id.clone(), ActionMsg {
                params: load_workflow_action_params(deps.storage, &workflow_id, &action_id).unwrap_or_default(),
                next_actions: action.next_actions.clone(),
                templates: load_workflow_action_templates(deps.storage, &workflow_id, &action_id).unwrap_or_default(),
                whitelisted_contracts: load_workflow_action_contracts(deps.storage, &workflow_id, &action_id).unwrap_or_default(),
            })).collect(),
        },
        publisher: workflow.publisher.clone(),
        state: workflow.state,
    } })
}

pub fn query_instances_by_requester(
    deps: Deps,
    requester_address: String,
) -> StdResult<GetInstancesResponse> {
    let requester = deps.api.addr_validate(&requester_address)?;
    let instances = load_workflow_instances_by_requester(deps.storage, &requester)?;
    Ok(GetInstancesResponse { 
        instances: instances.iter().map(|(instance_id, instance)| 
            to_workflow_instance_response(deps, &requester, instance_id, &instance)
    ).collect() })
}

pub fn query_workflow_instance(
    deps: Deps,
    user_address: String,
    instance_id: InstanceId,
) -> StdResult<GetWorkflowInstanceResponse> {
    let user_addr = deps.api.addr_validate(&user_address)?;
    let instance = load_workflow_instance(deps.storage, &user_addr, &instance_id)?;
    Ok(GetWorkflowInstanceResponse { instance: to_workflow_instance_response(deps, &user_addr, &instance_id, &instance) })
}

fn to_workflow_instance_response(deps: Deps, requester: &Addr, instance_id: &InstanceId, instance: &WorkflowInstance) -> WorkflowInstanceResponse {
    WorkflowInstanceResponse {
        base: NewInstanceMsg {
            workflow_id: instance.workflow_id.clone(),
            execution_type: instance.execution_type.clone(),
            expiration_time: instance.expiration_time,
            onchain_parameters: load_workflow_instance_params(deps.storage, &requester, &instance_id).unwrap_or_default(),
        },
        id: instance_id.clone(),
        state: instance.state.clone(),
        requester: requester.clone(),
        last_executed_action: instance.last_executed_action.clone(),
    }
}

pub fn query_user_payment_config(deps: Deps, user_address: String) -> StdResult<GetUserPaymentConfigResponse> {
    let user_addr = deps.api.addr_validate(&user_address)?;
    
    // Try to load existing config, or return None if not found
    let payment_config = match load_user_payment_config(deps.storage, &user_addr) {
        Ok(config) => Some(config),
        Err(_) => None,
    };
    
    Ok(GetUserPaymentConfigResponse { payment_config })
}