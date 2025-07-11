use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("This operation doesn't require funds")]
    InvalidFundsReceived {},

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("{0}")]
	GenericError(String) ,

    #[error("Template {template_id} is already approved")]
    TemplateAlreadyApproved {
        template_id: String,
    },

    #[error("Template {template_id} already exists")]
    TemplateAlreadyExists {
        template_id: String,
    },

    #[error("Template {template_id} not found")]
    TemplateNotFound {
        template_id: String,
    },

    #[error("Template {template_id} is not approved")]
    TemplateNotApproved {
        template_id: String,
    },

    #[error("Template {template_id} is private and can only be executed by its publisher")]
    TemplatePrivateAccessDenied {
        template_id: String,
    },

    #[error("Flow {flow_id} already exists")]
    FlowAlreadyExists {
        flow_id: String,
    },

    #[error("Flow {flow_id} not found")]
    FlowNotFound {
        flow_id: String,
    },

    #[error("Only the requester can cancel flow {flow_id}")]
    FlowCancelUnauthorized {
        flow_id: String,
    },

    #[error("Action {action_id} not found in template {template_id}")]
    ActionNotFound {
        template_id: String,
        action_id: String,
    },

    #[error("Denom {0} is not allowed for this action")]
    InvalidDenom(String),
}
