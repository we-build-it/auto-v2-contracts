use cosmwasm_std::{Deps, StdResult};
use crate::{
    msg::{FlowsResponse, TemplatesResponse, FlowResponse, TemplateResponse},
    state::{load_flow, load_template},
};

pub fn query_flows_by_requester(
    deps: Deps,
    requester_address: String,
) -> StdResult<FlowsResponse> {
    let requester = deps.api.addr_validate(&requester_address)?;
    let flows = crate::state::query_flows_by_requester(deps.storage, requester)?;
    Ok(FlowsResponse { flows })
}

pub fn query_templates_by_publisher(
    deps: Deps,
    publisher_address: String,
) -> StdResult<TemplatesResponse> {
    let publisher = deps.api.addr_validate(&publisher_address)?;
    let templates = crate::state::query_templates_by_publisher(deps.storage, publisher)?;
    Ok(TemplatesResponse { templates })
}

pub fn query_flow_by_id(deps: Deps, flow_id: String) -> StdResult<FlowResponse> {
    let flow = load_flow(deps.storage, &flow_id)?;
    Ok(FlowResponse { flow })
}

pub fn query_template_by_id(deps: Deps, template_id: String) -> StdResult<TemplateResponse> {
    let template = load_template(deps.storage, &template_id)?;
    Ok(TemplateResponse { template })
} 