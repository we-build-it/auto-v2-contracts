pub mod contract;
pub mod error;
pub mod events;
pub mod handlers;
pub mod helpers;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

pub const CONTRACT_NAME: &str = "crates.io:auto-fee-manager";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
