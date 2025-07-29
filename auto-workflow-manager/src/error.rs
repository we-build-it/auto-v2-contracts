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

    #[error("Workflow {workflow_id} is already approved")]
    WorkflowAlreadyApproved {
        workflow_id: String,
    },

    #[error("Workflow {workflow_id} already exists")]
    WorkflowAlreadyExists {
        workflow_id: String,
    },

    #[error("Workflow {workflow_id} not found")]
    WorkflowNotFound {
        workflow_id: String,
    },

    #[error("Workflow {workflow_id} is not approved")]
    WorkflowNotApproved {
        workflow_id: String,
    },

    #[error("Workflow {workflow_id} is private and can only be executed by its publisher")]
    PrivateWorkflowExecutionDenied {
        workflow_id: String,
    },

    #[error("Instance {instance_id} already exists")]
    InstanceAlreadyExists {
        instance_id: String,
    },

    #[error("Instance {instance_id} not found")]
    InstanceNotFound {
        instance_id: String,
    },

    #[error("Only the requester can do {action} on instance {instance_id}")]
    InstanceAccessUnauthorized {
        action: String,
        instance_id: String,
    },

    #[error("Action {action_id} not found in workflow {workflow_id}")]
    ActionNotFound {
        workflow_id: String,
        action_id: String,
    },

    #[error("Denom {0} is not allowed for this action")]
    InvalidDenom(String),
}
