use cosmwasm_std::{Deps, StdResult};
use crate::{
    msg::{GetInstancesResponse, GetWorkflowResponse, GetWorkflowInstanceResponse}, state::{get_workflow_instances_by_requester, load_workflow, load_workflow_instance},
};

pub fn query_instances_by_requester(
    deps: Deps,
    requester_address: String,
) -> StdResult<GetInstancesResponse> {
    let requester = deps.api.addr_validate(&requester_address)?;
    let flows = get_workflow_instances_by_requester(deps.storage, requester)?;
    Ok(GetInstancesResponse { flows: flows.values().cloned().collect() })
}

pub fn query_workflow_by_id(deps: Deps, template_id: String) -> StdResult<GetWorkflowResponse> {
    let template = load_workflow(deps.storage, &template_id)?;
    Ok(GetWorkflowResponse { template })
}

pub fn query_workflow_instance(
    deps: Deps,
    user_address: String,
    instance_id: u64,
) -> StdResult<GetWorkflowInstanceResponse> {
    let user_addr = deps.api.addr_validate(&user_address)?;
    let instance = load_workflow_instance(deps.storage, user_addr, instance_id)?;
    Ok(GetWorkflowInstanceResponse { instance })
} 