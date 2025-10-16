use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg, DepsMut, Deps, MessageInfo};

use crate::msg::ExecuteMsg;
use crate::error::ContractError;
use crate::state::{CONFIG, Config};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

/// Helper function to validate addresses
pub fn validate_address(deps: &DepsMut, address: &str, field_name: &str) -> Result<(), ContractError> {
    // Check for empty strings first (for specific error messages)
    if address.trim().is_empty() {
        return Err(ContractError::InvalidAddress { 
            reason: format!("{} cannot be empty", field_name) 
        });
    }
    // Validate address format
    deps.api.addr_validate(address).map_err(|_| ContractError::InvalidAddress { 
        reason: format!("{} is not a valid address", field_name) 
    })?;
    Ok(())
}

/// Helper function to check if an address is authorized
pub fn is_crank(deps: Deps, address: &Addr) -> Result<bool, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config.crank_authorized_address == *address)
}

/// Helper function to check if an address is the workflow manager
pub fn is_workflow_manager(deps: Deps, address: &Addr) -> Result<bool, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config.workflow_manager_address == Some(address.clone()))
}

/// Helper function to verify authorization for restricted functions
pub fn verify_crank(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    if !is_crank(deps, &info.sender)? {
        return Err(ContractError::NotAuthorized {
            address: info.sender.to_string(),
        });
    }
    Ok(())
}

/// Helper function to verify workflow manager authorization
pub fn verify_workflow_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    if !is_workflow_manager(deps, &info.sender)? {
        return Err(ContractError::NotAuthorized {
            address: info.sender.to_string(),
        });
    }
    Ok(())
}
