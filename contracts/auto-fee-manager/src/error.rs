use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Invalid address: {reason}")]
    InvalidAddress { reason: String },

    #[error("Address {address} is not authorized to execute this function")]
    NotAuthorized { address: String },

    #[error("No funds sent with deposit")]
    NoFundsSent {},

    #[error("Denom {denom} is not accepted for deposits")]
    DenomNotAccepted { denom: String },

    #[error("Withdrawal amount must be greater than zero")]
    InvalidWithdrawalAmount {},

    #[error("Insufficient balance. Available: {available}, Requested: {requested}")]
    InsufficientBalance { available: i128, requested: Uint128 },

    #[error("Invalid creator address: {reason}")]
    InvalidCreatorAddress { reason: String },

    #[error("Invalid fee type: {reason}")]
    InvalidFeeType { reason: String },

    #[error("Invalid payment: {reason}")]
    InvalidPayment { reason: String },

    #[error("No creator fees available to claim")]
    NoCreatorFeesToClaim {},

    #[error("No execution fees available to distribute")]
    NoExecutionFeesToDistribute {},

    #[error("No creator fees available to distribute")]
    NoCreatorFeesToDistribute {},

    #[error("Invalid max_debt: {reason}")]
    InvalidMaxDebt { reason: String },

    #[error("Accepted denoms cannot be empty")]
    EmptyAcceptedDenoms {},

    #[error("{0}")]
    Std(#[from] StdError),
}
