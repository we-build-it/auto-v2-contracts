use std::collections::HashSet;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Flow, Template};

#[cw_serde]
pub struct InstantiateMsg {
    pub approvers: HashSet<Addr>,
}


#[cw_serde]
pub struct ActionMsg {
    pub id: String,
    pub message_template: String,
    pub contract_address: Addr,
    pub allowed_denoms: HashSet<String>,
}

#[cw_serde]
pub struct TemplateMsg {
    pub id: String,
    pub publisher: Addr,
    pub actions: Vec<ActionMsg>,
    pub private: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    RequestForApproval {
        template: TemplateMsg,
    },
    ApproveTemplate {
        template_id: String,
    },
    RejectTemplate {
        template_id: String,
    },
    ExecuteFlow {
        flow_id: String,
        template_id: String,
        params: String,
    },
    CancelFlow {
        flow_id: String,
    },
}

#[cw_serde]
pub struct FlowsResponse {
    pub flows: Vec<Flow>,
}

#[cw_serde]
pub struct TemplatesResponse {
    pub templates: Vec<Template>,
}

#[cw_serde]
pub struct FlowResponse {
    pub flow: Flow,
}

#[cw_serde]
pub struct TemplateResponse {
    pub template: Template,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(FlowsResponse)]
    GetFlowsByRequester { requester_address: String },
    #[returns(TemplatesResponse)]
    GetTemplatesByPublisher { publisher_address: String },
    #[returns(FlowResponse)]
    GetFlowById { flow_id: String },
    #[returns(TemplateResponse)]
    GetTemplateById { template_id: String },
}

#[cw_serde]
pub struct MigrateMsg {
    // Add fields if you need to pass parameters during migration
}
