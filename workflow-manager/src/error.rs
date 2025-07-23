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
    WorkflowNotFound {
        template_id: String,
    },

    #[error("Template {template_id} is not approved")]
    WorkflowNotApproved {
        template_id: String,
    },

    #[error("Template {template_id} is private and can only be executed by its publisher")]
    PrivateWorkflowExecutionDenied {
        template_id: String,
    },

    #[error("Flow {flow_id} already exists")]
    InstanceAlreadyExists {
        flow_id: String,
    },

    #[error("Flow {flow_id} not found")]
    InstanceNotFound {
        flow_id: String,
    },

    #[error("Only the requester can do {action} on instance {instance_id}")]
    InstanceAccessUnauthorized {
        action: String,
        instance_id: String,
    },

    #[error("Action {action_id} not found in template {template_id}")]
    ActionNotFound {
        template_id: String,
        action_id: String,
    },

    #[error("Denom {0} is not allowed for this action")]
    InvalidDenom(String),
}
