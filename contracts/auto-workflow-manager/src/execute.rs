use std::{collections::HashMap, str::FromStr};

use cosmwasm_std::{to_json_string};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
    Reply, SubMsg
};

use auto_fee_manager::msg::ExecuteMsg as FeeManagerExecuteMsg;
use auto_fee_manager::msg::Fee as FeeManagerFee;
use auto_fee_manager::msg::FeeType as FeeManagerFeeType;
use auto_fee_manager::msg::UserFees as FeeManagerUserFees;

use crate::{
    msg::{
        ActionParamValue, ExecutionType, FeeType, FinishInstanceRequest, InstanceId, NewWorkflowMsg, UserFee, WorkflowInstanceState, WorkflowState, WorkflowVisibility
    },
    state::{load_config, load_user_payment_config},
    ContractError,
};
use cw_storage_plus::Map;

// Data structure to pass from execute to reply
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FeeEventData {
    pub user_address: String,
    pub fee_denom: String,
    pub fee_amount: Uint128,
    pub usd_amount: Uint128,
    pub debit_denom: String,
    pub debit_amount: Uint128,
    pub fee_type: FeeType,
    pub creator_address: Option<String>,
}

// Temporary storage for fee event data
pub const FEE_EVENT_DATA: Map<u64, Vec<FeeEventData>> = Map::new("fed");


use crate::{
    msg::{NewInstanceMsg, ParamId, TemplateId},
    state::{
        load_next_instance_id, load_workflow, load_workflow_action, load_workflow_action_params,
        load_workflow_action_template, load_workflow_instance, load_workflow_instance_params,
        remove_user_payment_config, remove_workflow_instance, save_user_payment_config,
        save_workflow, save_workflow_action, save_workflow_action_contracts,
        save_workflow_action_params, save_workflow_action_templates, save_workflow_instance,
        save_workflow_instance_params, validate_contract_is_whitelisted,
        validate_sender_is_action_executor, validate_sender_is_owner, validate_sender_is_publisher,
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
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/publish_workflow")
                .add_attribute("workflow_id", input_workflow.id)
                .add_attribute("publisher", info.sender.to_string())
                .add_attribute("state", new_workflow.state.to_string())
        ))
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
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/execute_instance")
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("workflow_id", new_instance.workflow_id)
                .add_attribute("requester", info.sender.to_string())
        ))
}

pub fn cancel_run(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64
) -> Result<Response, ContractError> {
    // Load the instance
    let instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    // Only cancel run if it's OneShot
    if matches!(instance.execution_type, ExecutionType::OneShot) {
        return Err(ContractError::GenericError(
            "Can't cancel run for one shot instances, use cancel_instance instead".to_string(),
        ));
    } else {
        let mut updated_instance = instance;
        // instance state remains Running, but last_executed_action is reset to None
        updated_instance.last_executed_action = None;
        save_workflow_instance(deps.storage, &info.sender, &instance_id, &updated_instance)?;
    }

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/cancel_run")
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("canceller", info.sender.to_string())
        ))
}

pub fn cancel_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_id: u64
) -> Result<Response, ContractError> {
    // Load the instance
    let instance =
        load_workflow_instance(deps.storage, &info.sender, &instance_id).map_err(|_| {
            ContractError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }
        })?;

    if matches!(instance.state, WorkflowInstanceState::Cancelled) || matches!(instance.state, WorkflowInstanceState::Finished) {
        return Err(ContractError::GenericError(
            "Can't cancel instance that is already cancelled or finished".to_string(),
        ));
    }

    let mut updated_instance = instance;
    updated_instance.state = WorkflowInstanceState::Cancelled;
    updated_instance.last_executed_action = None;
    save_workflow_instance(deps.storage, &info.sender, &instance_id, &updated_instance)?;

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/cancel_instance")
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("canceller", info.sender.to_string())
        ))
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
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/pause_schedule")
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("pauser", info.sender.to_string())
        ))
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
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/resume_schedule")
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("resumer", info.sender.to_string())
        ))
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

    // Instance must be running or finished (if recurrent)
    if !(matches!(user_instance.state, WorkflowInstanceState::Running) || 
        (matches!(user_instance.state, WorkflowInstanceState::Finished) && matches!(user_instance.execution_type, ExecutionType::Recurrent))) 
        {
        return Err(ContractError::GenericError(
            "Instance is not running".to_string(),
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
    
    // Update instance state based on whether this is an end action
    // if workflow.end_actions.contains(&action_id) {
    //     updated_instance.state = WorkflowInstanceState::Finished;
    // } else {
    //     updated_instance.state = WorkflowInstanceState::Running;
    // }
    
    save_workflow_instance(deps.storage, &user_addr, &instance_id, &updated_instance)?;

    Ok(Response::new()
        .add_messages(authz_msgs)
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/execute_action")
                .add_attribute("user_address", user_address)
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("action_id", action_id)
        ))
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

pub fn purge_instances(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instance_ids: Vec<u64>,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_owner(deps.storage, &info)?;

    let mut purged_instance_ids = Vec::new();
    let mut not_found_instance_ids = Vec::new();
    let mut not_purgued_instance_ids = Vec::new();

    for instance_id in instance_ids {
        let instance_result = load_workflow_instance(deps.storage, &info.sender, &instance_id);
        if instance_result.is_ok() {
            let instance = instance_result.unwrap();
            if matches!(instance.state, WorkflowInstanceState::Cancelled) || matches!(instance.state, WorkflowInstanceState::Finished) {
                purged_instance_ids.push(instance_id.to_string());
                remove_workflow_instance(deps.storage, &info.sender, &instance_id)?;
            } else {
                not_purgued_instance_ids.push(instance_id.to_string());
            }
        } else {
            not_found_instance_ids.push(instance_id.to_string());
        }
    }

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/purge_instances")
                .add_attribute("purged_instance_ids", purged_instance_ids.join(","))
                .add_attribute("not_found_instance_ids", not_found_instance_ids.join(","))
                .add_attribute("not_purgued_instance_ids", not_purgued_instance_ids.join(","))
        ))
}

pub fn set_user_payment_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    payment_config: PaymentConfig,
) -> Result<Response, ContractError> {
    // Validate user address
    let user_addr = deps.api.addr_validate(&info.sender.to_string())?;

    // Save the payment config
    save_user_payment_config(deps.storage, &user_addr, &payment_config)?;

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/set_user_payment_config")
                .add_attribute("user_address", info.sender.to_string())
                .add_attribute("payment_config", payment_config.to_string())
        ))
}

pub fn remove_user_payment_config_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Validate user address
    let user_addr = deps.api.addr_validate(&info.sender.to_string())?;

    // Remove the payment config
    remove_user_payment_config(deps.storage, &user_addr)?;

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/remove_user_payment_config")
                .add_attribute("user_address", info.sender.to_string())
        ))
}

pub fn override_prices_from_oracle(prices: HashMap<String, Decimal>) -> Result<HashMap<String, Decimal>, ContractError> {
    // TODO: obtain prices from oracle and override prices received from backend.
    //       for each everriden price, we need to emit an event with the denom and the price.
    Ok(prices)
}

pub fn charge_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    batch_id: String,
    prices_from_backend: HashMap<String, Decimal>,
    fees: Vec<UserFee>,
) -> Result<Response, ContractError> {
    validate_sender_is_owner(deps.storage, &info)?;

    let mut response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/charge_fees")
                .add_attribute("batch_id", batch_id.clone())
        );

    let prices = override_prices_from_oracle(prices_from_backend)?;

    // Load config to get fee manager address
    let config = load_config(deps.storage)?;

    // deps.querier.query_all_balances(&config.fee_manager_address)?;
    // Initialize reply ID counter starting from 100
    let mut reply_id = 1000u64;
    
    // Process fees for each user
    for user_fee in fees {
        // Get the payment config for this user
        let requester = deps.api.addr_validate(&user_fee.address)?;
        let payment_config = load_user_payment_config(deps.storage, &requester.clone())?;

        // Accumulate fees and funds for this user
        let mut accumulated_fees = Vec::new();
        let mut accumulated_funds = Vec::new();
        let mut accumulated_fee_events = Vec::new();
        let (use_wallet, mut current_usd_allowance) = match payment_config {
            PaymentConfig::Wallet { usd_allowance } => (true, usd_allowance),
            PaymentConfig::Prepaid => (false, Uint128::zero()),
        };

        for fee_total in &user_fee.totals {
            let denom_price = match prices.get(&fee_total.denom) {
                Some(price) => price,
                None => {
                    // Add error event for missing price
                    response = response.add_event(
                        cosmwasm_std::Event::new("autorujira-workflow-manager/fee-price-error")
                            .add_attribute("user_address", user_fee.address.clone())
                            .add_attribute("fee_denom", fee_total.denom.clone())
                            .add_attribute("error", "Price not found for fee denom")
                    );
                    continue; // Skip to next fee_total
                }
            };
            let usd_amount = (Decimal::from_atomics(fee_total.amount, 0).unwrap() * denom_price).to_uint_ceil();

            if use_wallet {
                if current_usd_allowance < usd_amount {
                    // If user has not enough allowance, set it to 0 and break as we can not charge any more fees
                    response = response.add_event(
                        cosmwasm_std::Event::new("autorujira-workflow-manager/fee-error")
                            .add_attribute("user_address", user_fee.address.clone())
                            .add_attribute("fee_denom", fee_total.denom.clone())
                            .add_attribute("fee_amount", fee_total.amount.to_string())
                            .add_attribute("usd_amount", usd_amount.to_string())
                            .add_attribute("error", "Not enough allowance")
                    );
                    current_usd_allowance = Uint128::zero();
                    break; // Break out of the loop as we can not charge any more fees for this user
                }
                current_usd_allowance -= usd_amount;
            }

            // We need to handle two cases, when fee_total.denom is debit_denom and when it's not
            let debit_denom_amount = if fee_total.denom == fee_total.debit_denom.clone() {
                fee_total.amount
            } else {
                let debit_denom_price = match prices.get(&fee_total.debit_denom) {
                    Some(price) => price,
                    None => {
                        // Add error event for missing price
                        response = response.add_event(
                            cosmwasm_std::Event::new("autorujira-workflow-manager/fee-price-error")
                                .add_attribute("user_address", user_fee.address.clone())
                                .add_attribute("debit_denom", fee_total.debit_denom.clone())
                                .add_attribute("error", "Price not found for debit denom")
                        );
                        continue; // Skip to next fee_total
                    }
                };
                (Decimal::from_atomics(fee_total.amount, 0).unwrap() * denom_price / debit_denom_price).to_uint_ceil()
            };

            // Only process if there's something to charge
            if debit_denom_amount > Uint128::zero() {
                let fee_manager_fee = FeeManagerFee {
                    fee_type: match fee_total.fee_type {
                        FeeType::Creator { instance_id } => {
                            let workflow_instance = load_workflow_instance(deps.storage, &requester.clone(), &instance_id)?;
                            let workflow = load_workflow(deps.storage, &workflow_instance.workflow_id)?;
                            FeeManagerFeeType::Creator { creator_address: workflow.publisher.clone() }
                        },
                        FeeType::Execution => FeeManagerFeeType::Execution,
                    },
                    denom: fee_total.debit_denom.clone(),
                    amount: debit_denom_amount.clone(),
                };
                accumulated_fees.push(fee_manager_fee.clone());

                if use_wallet {
                    accumulated_funds.push(cosmwasm_std::Coin {
                        amount: debit_denom_amount.clone(),
                        denom: fee_total.debit_denom.clone(),
                    });
                }

                // Create FeeEventData for this specific fee
                let fee_event_data = FeeEventData {
                    user_address: user_fee.address.clone(),
                    fee_denom: fee_total.denom.clone(),
                    fee_amount: fee_total.amount.clone(),
                    usd_amount: usd_amount.clone(),
                    debit_denom: fee_total.debit_denom.clone(),
                    debit_amount: debit_denom_amount.clone(),
                    fee_type: fee_total.fee_type.clone(),
                    creator_address: match fee_manager_fee.fee_type.clone() {
                        FeeManagerFeeType::Creator { creator_address } => Some(creator_address.to_string()),
                        _ => None,
                    },
                };
                accumulated_fee_events.push(fee_event_data);
            }
        }

        if use_wallet {
            save_user_payment_config(
                deps.storage,
                &requester.clone(),
                &PaymentConfig::Wallet { usd_allowance: current_usd_allowance }
            )?;
        }

        // Send single submessage per user if there are fees to process
        if !accumulated_fees.is_empty() {
            // Store accumulated fee event data for reply
            FEE_EVENT_DATA.save(deps.storage, reply_id, &accumulated_fee_events)?;

            // Send message to fee manager with reply
            if use_wallet {
                let fee_msg = FeeManagerExecuteMsg::ChargeFeesFromMessageCoins {
                    fees: accumulated_fees,
                };
                // Build AUTHZ message in name of requester
                let authz_msg = build_authz_execute_contract_msg(
                    &_env,
                    &requester.clone(),
                    &config.fee_manager_address,
                    &to_json_string(&fee_msg)?,
                    &accumulated_funds,
                )?;
                let sub_msg = SubMsg::reply_always(authz_msg, reply_id);
                response = response.add_submessage(sub_msg);
            } else {
                let fee_msg = FeeManagerExecuteMsg::ChargeFeesFromUserBalance {
                    batch: vec![FeeManagerUserFees {
                        user: requester.clone(),
                        fees: accumulated_fees,
                    }],
                };
                // Direct call to fee manager
                let wasm_msg = WasmMsg::Execute {
                    contract_addr: config.fee_manager_address.to_string(),
                    msg: to_json_binary(&fee_msg)?,
                    funds: vec![],
                };
                let sub_msg = SubMsg::reply_always(wasm_msg, reply_id);
                response = response.add_submessage(sub_msg);
            }
            // Increment reply_id for next user
            reply_id += 1;
        }
    }

    Ok(response)
}


/// Handle reply from fee manager contract
pub fn handle_fee_manager_reply(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    // Check if there was an error
    if let cosmwasm_std::SubMsgResult::Err(error_msg) = reply.result {
        // Load fee event data from storage for error event
        let fee_event_data_vec = FEE_EVENT_DATA.load(deps.storage, reply.id)?;
        
        let mut response = Response::new();
        
        // Emit error event for each fee
        for fee_event_data in fee_event_data_vec {
            response = response.add_event(
                cosmwasm_std::Event::new("autorujira-workflow-manager/fee-error")
                    .add_attribute("user_address", fee_event_data.user_address)
                    .add_attribute("denom", fee_event_data.fee_denom)
                    .add_attribute("amount", fee_event_data.fee_amount.to_string())
                    .add_attribute("usd_amount", fee_event_data.usd_amount.to_string())
                    .add_attribute("debit_denom", fee_event_data.debit_denom)
                    .add_attribute("debit_amount", fee_event_data.debit_amount.to_string())
                    .add_attribute("fee_type", fee_event_data.fee_type.to_string())
                    .add_attribute("error", "true")
                    .add_attribute("details", error_msg.clone())
            );
        }
        
        // Clean up the temporary data
        FEE_EVENT_DATA.remove(deps.storage, reply.id);
        
        return Ok(response);
    }
    
    // Success case - extract the reply data
    let _reply_data = reply.result.into_result().map_err(|_| {
        ContractError::GenericError("Fee manager contract execution failed".to_string())
    })?;
    
    // Load fee event data from storage
    let fee_event_data_vec = FEE_EVENT_DATA.load(deps.storage, reply.id)?;
    
    let mut response = Response::new();
    
    // Emit fee-charged event for each fee
    for fee_event_data in fee_event_data_vec {
        response = response.add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/fee-charged")
                .add_attribute("user_address", fee_event_data.user_address)
                .add_attribute("denom", fee_event_data.fee_denom)
                .add_attribute("amount", fee_event_data.fee_amount.to_string())
                .add_attribute("usd_amount", fee_event_data.usd_amount.to_string())
                .add_attribute("debit_denom", fee_event_data.debit_denom)
                .add_attribute("debit_amount", fee_event_data.debit_amount.to_string())
                .add_attribute("fee_type", fee_event_data.fee_type.to_string())
                .add_attribute("creator_address", fee_event_data.creator_address.unwrap_or_default()));
    }
    
    // Clean up the temporary data
    FEE_EVENT_DATA.remove(deps.storage, reply.id);
    
    Ok(response)
}

pub fn finish_instances(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instances: Vec<FinishInstanceRequest>,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_owner(deps.storage, &info)?;

    let mut finished_instance_ids = Vec::new();
    let mut not_found_instance_ids = Vec::new();
    let mut already_finished_instance_ids = Vec::new();

    for request in instances {
        let requester = deps.api.addr_validate(&request.requester)?;
        
        for instance_id in request.instance_ids {
            // Load instance - O(1) access
            match load_workflow_instance(deps.storage, &requester, &instance_id) {
                Ok(instance) => {
                    // Check if instance is already finished
                    if matches!(instance.state, WorkflowInstanceState::Finished) {
                        already_finished_instance_ids.push(instance_id.to_string());
                        continue;
                    }

                    // Update instance state to Finished
                    let mut updated_instance = instance;
                    updated_instance.state = WorkflowInstanceState::Finished;
                    save_workflow_instance(deps.storage, &requester, &instance_id, &updated_instance)?;
                    
                    finished_instance_ids.push(instance_id.to_string());
                }
                Err(_) => {
                    not_found_instance_ids.push(instance_id.to_string());
                }
            }
        }
    }

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/finish_instances")
                .add_attribute("finished_instance_ids", finished_instance_ids.join(","))
                .add_attribute("not_found_instance_ids", not_found_instance_ids.join(","))
                .add_attribute("already_finished_instance_ids", already_finished_instance_ids.join(","))
        ))
}

pub fn reset_instance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_address: String,
    instance_id: InstanceId,
) -> Result<Response, ContractError> {
    // Validate sender is admin
    validate_sender_is_owner(deps.storage, &info)?;

    // Validate user address
    let user_addr = deps.api.addr_validate(&user_address)?;

    // Load the instance
    let instance = load_workflow_instance(deps.storage, &user_addr, &instance_id).map_err(|_| {
        ContractError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        }
    })?;

    let mut updated_instance = instance;
    
    // Handle different execution types
    if matches!(updated_instance.execution_type, ExecutionType::OneShot) {
        // For OneShot instances, change state to Finished
        updated_instance.state = WorkflowInstanceState::Finished;
    } else {
        // For Recurrent instances, reset last_executed_action to None
        updated_instance.last_executed_action = None;
    }
    
    save_workflow_instance(deps.storage, &user_addr, &instance_id, &updated_instance)?;

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-workflow-manager/reset_instance")
                .add_attribute("user_address", user_address)
                .add_attribute("instance_id", instance_id.to_string())
                .add_attribute("execution_type", match updated_instance.execution_type {
                    ExecutionType::OneShot => "oneshot",
                    ExecutionType::Recurrent => "recurrent",
                })
        ))
}
