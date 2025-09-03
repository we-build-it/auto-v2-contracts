use std::{collections::HashMap, str::FromStr};

use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg
};

use crate::{
    msg::{
        ActionParamValue, ExecutionType, FeeType, NewWorkflowMsg, UserFee, WorkflowInstanceState,
        WorkflowState, WorkflowVisibility,
    },
    state::{load_user_payment_config, load_config, PaymentSource},
    ContractError,
};
use crate::{
    msg::{NewInstanceMsg, ParamId, TemplateId},
    state::{
        load_next_instance_id, load_workflow, load_workflow_action, load_workflow_action_params,
        load_workflow_action_template, load_workflow_instance, load_workflow_instance_params,
        remove_user_payment_config, remove_workflow_instance, save_user_payment_config,
        save_workflow, save_workflow_action, save_workflow_action_contracts,
        save_workflow_action_params, save_workflow_action_templates, save_workflow_instance,
        save_workflow_instance_params, validate_contract_is_whitelisted,
        validate_sender_is_action_executor, validate_sender_is_admin, validate_sender_is_publisher,
        Action, PaymentConfig, Workflow, WorkflowInstance,
    },
    utils::build_authz_execute_contract_msg,
};

pub fn publish_workflow(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    input_workflow: NewWorkflowMsg,
) -> Result<Response, ContractError> {
    validate_sender_is_publisher(deps.storage, &info)?;

    // Check if workflow already exists
    if load_workflow(deps.storage, &input_workflow.id).is_ok() {
        return Err(ContractError::WorkflowAlreadyExists {
            workflow_id: input_workflow.id.clone(),
        });
    }

    let new_workflow = Workflow {
        start_actions: input_workflow.start_actions,
        end_actions: input_workflow.end_actions,
        visibility: input_workflow.visibility,
        publisher: info.sender.clone(),
        state: WorkflowState::Approved,
    };

    save_workflow(deps.storage, &input_workflow.id, &new_workflow)?;
    for (action_id, action) in input_workflow.actions {
        let new_action = Action {
            next_actions: action.next_actions,
        };
        save_workflow_action(deps.storage, &input_workflow.id, &action_id, &new_action)?;
        save_workflow_action_params(deps.storage, &input_workflow.id, &action_id, &action.params)?;
        save_workflow_action_templates(
            deps.storage,
            &input_workflow.id,
            &action_id,
            &action.templates,
        )?;
        save_workflow_action_contracts(
            deps.storage,
            &input_workflow.id,
            &action_id,
            &action.whitelisted_contracts,
        )?;
    }

    Ok(Response::new()
        .add_attribute("method", "publish_workflow")
        .add_attribute("workflow_id", input_workflow.id)
        .add_attribute("publisher", info.sender.to_string())
        .add_attribute("state", new_workflow.state.to_string()))
}

pub fn execute_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance: NewInstanceMsg,
) -> Result<Response, ContractError> {
    // Check if workflow exists
    let workflow = load_workflow(deps.storage, &instance.workflow_id).map_err(|_| {
        ContractError::WorkflowNotFound {
            workflow_id: instance.workflow_id.clone(),
        }
    })?;

    // Check if workflow is approved
    if !matches!(workflow.state, WorkflowState::Approved) {
        return Err(ContractError::WorkflowNotApproved {
            workflow_id: instance.workflow_id.clone(),
        });
    }

    // Check if workflow is private and sender is not the publisher
    if matches!(workflow.visibility, WorkflowVisibility::Private)
        && info.sender != workflow.publisher
    {
        return Err(ContractError::PrivateWorkflowExecutionDenied {
            workflow_id: instance.workflow_id.clone(),
        });
    }

    // Generate auto-incremental ID for the instance
    let instance_id = load_next_instance_id(deps.storage)?;

    // Set initial state
    let new_instance: WorkflowInstance = WorkflowInstance {
        workflow_id: instance.workflow_id,
        state: WorkflowInstanceState::Running,
        last_executed_action: None,
        execution_type: instance.execution_type,
        expiration_time: instance.expiration_time,
        // requester: info.sender.clone(),
    };

    // Save the instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &new_instance)?;
    save_workflow_instance_params(
        deps.storage,
        &info.sender,
        &instance_id,
        &instance.onchain_parameters,
    )?;

    Ok(Response::new()
        .add_attribute("method", "execute_instance")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("workflow_id", new_instance.workflow_id)
        .add_attribute("requester", info.sender.to_string()))
}

pub fn cancel_run(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
    run_id: String,
) -> Result<Response, ContractError> {
    // Load the instance
    let instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Only remove the instance if it's OneShot
    if matches!(instance.execution_type, ExecutionType::OneShot) {
        remove_workflow_instance(deps.storage, &info.sender, &instance_id)?;
    }

    Ok(Response::new()
        .add_attribute("method", "cancel_run")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("run_id", run_id)
        .add_attribute("canceller", info.sender.to_string()))
}

pub fn cancel_schedule(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Check if instance is NOT Recurrent (only Recurrent instances can have their schedule changed)
    if !matches!(instance.execution_type, ExecutionType::Recurrent) {
        return Err(ContractError::GenericError(
            "Can't change schedule for non-recurrent instances".to_string(),
        ));
    }

    // Remove the instance
    remove_workflow_instance(deps.storage, &info.sender, &instance_id)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_schedule")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("canceller", info.sender.to_string()))
}

pub fn pause_schedule(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let mut instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Check if instance is NOT Recurrent (only Recurrent instances can have their schedule changed)
    if !matches!(instance.execution_type, ExecutionType::Recurrent) {
        return Err(ContractError::GenericError(
            "Can't change schedule for non-recurrent instances".to_string(),
        ));
    }

    // Check if instance is running
    if !matches!(instance.state, WorkflowInstanceState::Running) {
        return Err(ContractError::GenericError(
            "Instance is not running".to_string(),
        ));
    }

    // Pause the instance
    instance.state = WorkflowInstanceState::Paused;

    // Save the updated instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &instance)?;

    Ok(Response::new()
        .add_attribute("method", "pause_schedule")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("pauser", info.sender.to_string()))
}

pub fn resume_schedule(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64,
) -> Result<Response, ContractError> {
    // Load the instance
    let mut instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Check if instance is NOT Recurrent (only Recurrent instances can have their schedule changed)
    if !matches!(instance.execution_type, ExecutionType::Recurrent) {
        return Err(ContractError::GenericError(
            "Can't change schedule for non-recurrent instances".to_string(),
        ));
    }

    // Check if instance is paused
    if !matches!(instance.state, WorkflowInstanceState::Paused) {
        return Err(ContractError::GenericError(
            "Instance is not paused".to_string(),
        ));
    }

    // Resume the instance
    instance.state = WorkflowInstanceState::Running;

    // Save the updated instance
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &instance)?;

    Ok(Response::new()
        .add_attribute("method", "resume_schedule")
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("resumer", info.sender.to_string()))
}

pub fn execute_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    instance_id: u64,
    action_id: String,
    template_id: TemplateId,
    params: Option<HashMap<String, ActionParamValue>>,
) -> Result<Response, ContractError> {
    // Validate sender is action executor
    validate_sender_is_action_executor(deps.storage, &info)?;

    // Load user instance
    let user_addr = deps.api.addr_validate(&user_address)?;
    let user_instance: WorkflowInstance =
        load_workflow_instance(deps.storage, &user_addr, &instance_id)?;

    // Validate instance expiration time
    if env.block.time >= user_instance.expiration_time {
        return Err(ContractError::GenericError(
            "Instance has expired".to_string(),
        ));
    }

    // Load workflow from user_instance.workflow_id
    let workflow = load_workflow(deps.storage, &user_instance.workflow_id)?;

    // Ensure the action exists
    let _action_to_execute =
        load_workflow_action(deps.storage, &user_instance.workflow_id, &action_id).map_err(
            |_| ContractError::ActionNotFound {
                workflow_id: user_instance.workflow_id.clone(),
                action_id: action_id.clone(),
            },
        )?;

    let can_execute = match &user_instance.last_executed_action {
        None => workflow.start_actions.contains(&action_id),
        Some(last_executed_action_id) => {
            let last_executed_action = load_workflow_action(
                deps.storage,
                &user_instance.workflow_id,
                &last_executed_action_id,
            )
            .map_err(|_| ContractError::ActionNotFound {
                workflow_id: user_instance.workflow_id.clone(),
                action_id: last_executed_action_id.clone(),
            })?;
            last_executed_action.next_actions.contains(&action_id)
                || (user_instance.execution_type == ExecutionType::Recurrent
                    && workflow.end_actions.contains(last_executed_action_id)
                    && workflow.start_actions.contains(&action_id))
        }
    };

    if !can_execute {
        return Err(ContractError::GenericError(
            "Action cannot be executed: not first execution, not valid next action, and not recurrent start action".to_string()
        ));
    }

    // Get action parameters and create new HashMap
    let action_params =
        load_workflow_action_params(deps.storage, &user_instance.workflow_id, &action_id)?;
    let instance_params = load_workflow_instance_params(deps.storage, &user_addr, &instance_id)?;
    let mut resolved_params = HashMap::<String, ActionParamValue>::new();

    for (key, value) in action_params {
        // si param.value es #ip.requester => busco user_instance.requester
        // si param.value comienza con #ip, busco en user_instance.params
        // si param.value comienza con #cp, busco en execute_action_params
        // else es un valor fijo
        let resolved_value = resolve_param_value(&value, &user_addr, &instance_params, &params)?;
        resolved_params.insert(key.clone(), resolved_value);
    }

    // Execute template-based action
    let msgs: Vec<WasmMsg> = execute_dynamic_template(
        deps.storage,
        &user_instance.workflow_id,
        &action_id,
        &template_id,
        &resolved_params,
        &params,
    )?;

    let authz_msgs: Vec<CosmosMsg> = msgs
        .iter()
        .map(|msg| {
            let cosmos_msg = match msg {
                WasmMsg::Execute {
                    contract_addr,
                    msg,
                    funds,
                } => {
                    build_authz_execute_contract_msg(
                        &env,
                        &user_addr,
                        &deps.api.addr_validate(contract_addr)?,
                        &String::from_utf8(msg.to_vec()).unwrap(),
                        &funds,
                    )
                }
                _ => {
                    return Err(ContractError::GenericError(
                        "Unsupported message type".to_string(),
                    ))
                }
            };
            cosmos_msg
        })
        .collect::<Result<Vec<CosmosMsg>, ContractError>>()?;

    // Update instance with last executed action
    let mut updated_instance = user_instance;
    updated_instance.last_executed_action = Some(action_id.clone());
    save_workflow_instance(deps.storage, &user_addr, &instance_id, &updated_instance)?;

    Ok(Response::new()
        .add_messages(authz_msgs)
        .add_attribute("method", "execute_action")
        .add_attribute("user_address", user_address)
        .add_attribute("instance_id", instance_id.to_string())
        .add_attribute("action_id", action_id))
}

fn resolve_param_value(
    param_value: &ActionParamValue,
    user_addr: &Addr,
    instance_params: &HashMap<ParamId, ActionParamValue>,
    execute_action_params: &Option<HashMap<ParamId, ActionParamValue>>,
) -> Result<ActionParamValue, ContractError> {
    let value_str = match param_value {
        ActionParamValue::String(s) => s,
        ActionParamValue::BigInt(s) => s,
    };

    if value_str == "#ip.requester" {
        Ok(ActionParamValue::String(user_addr.to_string()))
    } else if value_str.starts_with("#ip.") {
        // Extract the key after #ip.
        let key = &value_str[4..];
        if let Some(value) = instance_params.get(key) {
            Ok(value.clone())
        } else {
            Err(ContractError::GenericError(format!(
                "Parameter '{}' not found in instance parameters",
                key
            )))
        }
    } else if value_str.starts_with("#cp.") {
        // Extract the key after #cp.
        let key = &value_str[4..];
        if let Some(params) = execute_action_params {
            if let Some(value) = params.get(key) {
                Ok(value.clone())
            } else {
                Err(ContractError::GenericError(format!(
                    "Parameter '{}' not found in execute action parameters",
                    key
                )))
            }
        } else {
            Err(ContractError::GenericError(
                "Execute action parameters not provided".to_string(),
            ))
        }
    } else {
        Ok(param_value.clone()) // Fixed value
    }
}

//=========== DYNAMIC TEMPLATE ACTION ============
fn execute_dynamic_template(
    storage: &dyn cosmwasm_std::Storage,
    workflow_id: &str,
    action_id: &str,
    template_id: &TemplateId,
    resolved_params: &HashMap<String, ActionParamValue>,
    execute_action_params: &Option<HashMap<String, ActionParamValue>>,
) -> Result<Vec<WasmMsg>, ContractError> {
    // Load template for this action
    let template = load_workflow_action_template(
        storage,
        &workflow_id.to_string(),
        &action_id.to_string(),
        &template_id.to_string(),
    )
    .map_err(|_| ContractError::TemplateNotFound {
        workflow_id: workflow_id.to_string(),
        action_id: action_id.to_string(),
        template_id: template_id.to_string(),
    })?;

    // Resolve template parameters
    let resolved_contract =
        resolve_template_parameter(&template.contract, resolved_params, execute_action_params)?;
    let resolved_message =
        resolve_template_parameter(&template.message, resolved_params, execute_action_params)?;
    let resolved_funds =
        resolve_template_funds(&template.funds, resolved_params, execute_action_params)?;

    // Validate that the resolved contract is whitelisted
    validate_contract_is_whitelisted(
        storage,
        &workflow_id.to_string(),
        &action_id.to_string(),
        &resolved_contract,
    )?;

    // Create the WasmMsg
    let wasm_msg = WasmMsg::Execute {
        contract_addr: resolved_contract,
        msg: Binary::from(resolved_message.as_bytes()),
        funds: resolved_funds,
    };

    Ok(vec![wasm_msg])
}

fn resolve_template_parameter(
    template_param: &str,
    resolved_params: &HashMap<String, ActionParamValue>,
    execute_action_params: &Option<HashMap<String, ActionParamValue>>,
) -> Result<String, ContractError> {
    let mut result = template_param.to_string();

    // Replace {{param}} placeholders with resolved values
    for (key, value) in resolved_params {
        let placeholder = format!("{{{{{}}}}}", key);
        let value_str = match value {
            ActionParamValue::String(s) => s.clone(),
            ActionParamValue::BigInt(s) => s.clone(),
        };
        result = result.replace(&placeholder, &value_str);
    }

    // Replace #cp.param placeholders with execute action params
    if let Some(params) = execute_action_params {
        for (key, value) in params {
            let placeholder = format!("#cp.{}", key);
            let value_str = match value {
                ActionParamValue::String(s) => s.clone(),
                ActionParamValue::BigInt(s) => s.clone(),
            };
            result = result.replace(&placeholder, &value_str);
        }
    }

    Ok(result)
}

fn resolve_template_funds(
    template_funds: &[(String, String)],
    resolved_params: &HashMap<String, ActionParamValue>,
    execute_action_params: &Option<HashMap<String, ActionParamValue>>,
) -> Result<Vec<cosmwasm_std::Coin>, ContractError> {
    let mut resolved_funds = Vec::new();

    for (amount_template, denom_template) in template_funds {
        let resolved_amount =
            resolve_template_parameter(amount_template, resolved_params, execute_action_params)?;
        let resolved_denom =
            resolve_template_parameter(denom_template, resolved_params, execute_action_params)?;

        let amount = Uint128::from_str(&resolved_amount)?;
        resolved_funds.push(cosmwasm_std::Coin {
            amount,
            denom: resolved_denom,
        });
    }

    Ok(resolved_funds)
}

//=========== DYNAMIC TEMPLATE ACTION (END) ============

pub fn set_user_payment_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_address: String,
    payment_config: PaymentConfig,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_admin(deps.storage, &info)?;

    // Validate user address
    let user_addr = deps.api.addr_validate(&user_address)?;

    // Save the payment config
    save_user_payment_config(deps.storage, &user_addr, &payment_config)?;

    Ok(Response::new()
        .add_attribute("method", "set_user_payment_config")
        .add_attribute("user_address", user_address)
        .add_attribute("allowance_usd", payment_config.allowance_usd.to_string())
        .add_attribute("source", format!("{:?}", payment_config.source)))
}

pub fn remove_user_payment_config_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_address: String,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_admin(deps.storage, &info)?;

    // Validate user address
    let user_addr = deps.api.addr_validate(&user_address)?;

    // Remove the payment config
    remove_user_payment_config(deps.storage, &user_addr)?;

    Ok(Response::new()
        .add_attribute("method", "remove_user_payment_config")
        .add_attribute("user_address", user_address))
}

pub fn charge_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    batch_id: String,
    fees: Vec<UserFee>,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_admin(deps.storage, &info)?;

    let mut response = Response::new()
        .add_attribute("method", "charge_fees")
        .add_attribute("batch_id", batch_id.clone())
        .add_attribute("users_count", fees.len().to_string());

    // TODO: Implement fee charging logic
    // This will involve:
    // 1. Validating user payment configs
    // 2. Checking allowances
    // 3. Processing payments through external service
    // 4. Updating user payment states

    // Collect unique denominations to emit rate events
    let mut unique_denoms = std::collections::HashSet::new();
    for user_fee in &fees {
        for fee_total in &user_fee.totals {
            unique_denoms.insert(fee_total.denom.clone());
        }
    }

    // Get quotes for all unique denominations
    let quotes = get_quotes(&unique_denoms);

    // Emit rate events for each unique denomination
    for denom in &unique_denoms {
        let quote = quotes.get(denom).unwrap();
        let usd_rate = convert_quote_to_usd_decimal(*quote);

        response = response.add_event(
            cosmwasm_std::Event::new("fee-rate")
                .add_attribute("denom", denom.clone())
                .add_attribute("usd_rate", usd_rate.to_string()),
        );
    }

    // Emit events for each fee (without rate info)
    for user_fee in fees {
        // Get the payment config for this user
        let requester = deps.api.addr_validate(&user_fee.address)?;
        let payment_config = load_user_payment_config(deps.storage, &requester)?;

        for fee_total in &user_fee.totals {
            // Get the quote for this denomination
            let quote = quotes.get(&fee_total.denom).unwrap();

            // TODO: quote not found
            
            // Simplified calculation to avoid Decimal precision issues
            let fee_total_amount_usd = (fee_total.amount * quote) / Uint128::new(10_u128.pow(fee_total.denom_decimals as u32));

            // Get the workflow instance to extract the executor
            let workflow_instance =
                load_workflow_instance(deps.storage, &requester, &fee_total.instance_id)?;
            let workflow = load_workflow(deps.storage, &workflow_instance.workflow_id)?;

            let creator = workflow.publisher.clone();

            // Get user's allowance in USD (already in utoken with 8 decimals)
            let user_allowance_usd = payment_config.allowance_usd;

            let (amount_charged, amount_charged_usd) = 
            // Case 1: allowance >= fee_total.amount_usd
            if user_allowance_usd >= fee_total_amount_usd
            {
                (fee_total.amount, fee_total_amount_usd)
            }
            // Case 2: 0 < allowance < fee_total.amount_usd => charge what allowance represents
            else if user_allowance_usd > Uint128::zero() {
                // Simplified calculation: convert allowance USD to denom amount
                let amount_to_charge = (user_allowance_usd * Uint128::new(10_u128.pow(fee_total.denom_decimals as u32))) / quote;

                (amount_to_charge, user_allowance_usd)
            }
            // Case 3: allowance = 0 => charge nothing
            else {
                (Uint128::zero(), Uint128::zero())
            };

            // Update user's allowance
            let new_allowance = user_allowance_usd - amount_charged_usd;
            save_user_payment_config(
                deps.storage,
                &requester,
                &PaymentConfig {
                    allowance_usd: new_allowance,
                    source: payment_config.source.clone(),
                },
            )?;

            // Load config to get fee manager address
            let config = load_config(deps.storage)?;
            
            // Helper function to create fee message
            let create_fee_message = |fee_type: &str, creator_address: Option<&Addr>| -> serde_json::Value {
                let fee_data = {
                    let mut fee = serde_json::json!({
                        "timestamp": _env.block.time.seconds(),
                        "amount": amount_charged.to_string(),
                        "denom": fee_total.denom,
                        "fee_type": fee_type
                    });
                    
                    if let Some(creator) = creator_address {
                        fee["creator_address"] = serde_json::Value::String(creator.to_string());
                    } else {
                        fee["creator_address"] = serde_json::Value::Null;
                    }
                    
                    fee
                };
                
                match payment_config.source {
                    PaymentSource::Wallet => {
                        serde_json::json!({
                            "charge_fees_from_message_coins": {
                                "fees": [fee_data]
                            }
                        })
                    },
                    PaymentSource::Prepaid => {
                        serde_json::json!({
                            "charge_fees_from_user_balance": {
                                "batch": [{
                                    "user": requester.to_string(),
                                    "fees": [fee_data]
                                }]
                            }
                        })
                    }
                }
            };
            
            // Create fee message based on fee type
            let fee_msg = match fee_total.fee_type {
                FeeType::Execution => create_fee_message("execution", None),
                FeeType::Creator => create_fee_message("creator", Some(&creator)),
            };
            
            // Send message to fee manager
            match payment_config.source {
                PaymentSource::Wallet => {
                    // Build AUTHZ message in name of requester
                    let authz_msg = build_authz_execute_contract_msg(
                        &_env,
                        &requester,
                        &config.fee_manager_address,
                        &fee_msg.to_string(),
                        &vec![],
                    )?;
                    
                    response = response.add_message(authz_msg);
                },
                PaymentSource::Prepaid => {
                    // Direct call to fee manager
                    let wasm_msg = WasmMsg::Execute {
                        contract_addr: config.fee_manager_address.to_string(),
                        msg: to_json_binary(&fee_msg)?,
                        funds: vec![],
                    };
                    
                    response = response.add_message(wasm_msg);
                },
            }

            response = response.add_event(
                cosmwasm_std::Event::new("fee-charged")
                    .add_attribute("user_address", user_fee.address.clone())
                    .add_attribute("denom", fee_total.denom.clone())
                    .add_attribute("amount_charged", amount_charged.to_string())
                    .add_attribute("fee_type", fee_total.fee_type.to_string())
                    .add_attribute("instance_id", fee_total.instance_id.to_string()),
            );
        }
    }

    Ok(response)
}

/// Mock function to get USD quotes for different denominations
/// TODO: Replace with actual oracle integration
fn get_quotes(denoms: &std::collections::HashSet<String>) -> HashMap<String, Uint128> {
    let mut quotes = HashMap::new();

    for denom in denoms {
        // Mock: all tokens quote at 0.5 USD
        // In real implementation, this would fetch from oracle
        // Oracle returns 8 decimal precision, so 0.5 USD = 50000000
        quotes.insert(denom.clone(), Uint128::new(50000000));
    }

    quotes
}

/// Convert oracle quote to USD decimal value
/// Oracle returns 8 decimal precision, so we need to convert to Decimal
fn convert_quote_to_usd_decimal(quote: Uint128) -> Decimal {
    // Oracle returns 8 decimal precision (e.g., 50000000 for 0.5 USD)
    // Convert to Decimal with 8 decimal places
    Decimal::from_ratio(quote, Uint128::new(100000000))
}
