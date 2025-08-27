use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo,
    Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MessageCountResponse, QueryMsg};
use crate::state::{get_message_count, increment_message_count, Config, CONFIG};

pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB limit

pub fn instantiate(
    deps: DepsMut,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;
    
    let config = Config { admin: admin.clone() };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Echo { message, attributes } => {
            execute_echo(deps, info, message, attributes)
        }
    }
}

fn execute_echo(
    deps: DepsMut,
    info: MessageInfo,
    message: Binary,
    custom_attributes: Vec<(String, String)>,
) -> Result<Response, ContractError> {
    // Validate message size
    if message.len() > MAX_MESSAGE_SIZE {
        return Err(ContractError::MessageTooLarge {});
    }

    // Increment message count
    let message_count = increment_message_count(deps.storage);

    // Create event
    let mut event = Event::new("echo_message")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("message_count", message_count.to_string())
        .add_attribute("message_size", message.len().to_string());

    // Add custom attributes if provided
    for (key, value) in custom_attributes {
        event = event.add_attribute(key, value);
    }

    // Add message hash as attribute for reference
    let message_hash = format!("{:x}", md5::compute(&message));
    event = event.add_attribute("message_hash", message_hash);

    // Return all funds received
    let mut response = Response::new()
        .add_event(event)
        .add_attribute("method", "echo")
        .add_attribute("processed", "true");

    // Add bank send messages to return all funds to sender
    if !info.funds.is_empty() {
        let sender = info.sender.to_string();
        for coin in &info.funds {
            response = response.add_message(
                cosmwasm_std::BankMsg::Send {
                    to_address: sender.clone(),
                    amount: vec![coin.clone()],
                }
            );
        }
    }

    Ok(response)
}

pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::MessageCount {} => to_json_binary(&query_message_count(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
    })
}

fn query_message_count(deps: Deps) -> StdResult<MessageCountResponse> {
    let count = get_message_count(deps.storage);
    Ok(MessageCountResponse { count })
}
