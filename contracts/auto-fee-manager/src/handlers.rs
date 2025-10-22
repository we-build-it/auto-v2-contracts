use crate::events::{balance_below_threshold, deposit_completed};
use crate::helpers::{verify_crank, verify_workflow_manager};
use crate::msg::{AcceptedDenomValue, Fee, FeeType};
use crate::state::{
    CONFIG, USER_BALANCES, CREATOR_FEES, EXECUTION_FEES, DISTRIBUTION_FEES, DEPOSIT_ACCEPTED_DENOMS,
    SUBSCRIBED_CREATORS,
};
use crate::{error::ContractError, msg::UserFees};
use cosmwasm_std::{
    Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw_utils::nonpayable;
use hashbrown::HashMap;
use std::collections::HashSet;

pub fn handle_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Validate that funds were sent
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    // Track denoms that turned positive
    let mut balances_turned_positive = Vec::new();

    // Process each coin sent
    for coin in &info.funds {
        // Validate that the denom is accepted
        DEPOSIT_ACCEPTED_DENOMS
            .may_load(deps.storage, coin.denom.as_str())? // StdResult<Option<_>>
            .ok_or_else(|| ContractError::DenomNotAccepted {
                denom: coin.denom.clone(),
            })?;

        // Get current balance for this user and denom
        let current_balance = USER_BALANCES
            .may_load(deps.storage, (info.sender.clone(), coin.denom.as_str()))?
            .unwrap_or(0);

        // Calculate new balance (can be negative for debt)
        let new_balance = current_balance + coin.amount.u128() as i128;

        // Check if balance turned positive (was negative, now positive or zero)
        if current_balance < 0 && new_balance >= 0 {
            balances_turned_positive.push(coin.denom.clone());
        }

        // Save the new balance
        USER_BALANCES.save(
            deps.storage,
            (info.sender.clone(), coin.denom.as_str()),
            &new_balance,
        )?;
    }

    // Create response
    let mut response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/deposit")
                .add_attribute("user", info.sender.to_string())
                .add_attribute("funds", format!("{:?}", info.funds))
        );

    // Add event only if any balances turned positive
    if !balances_turned_positive.is_empty() {
        response = response.add_event(deposit_completed(&info.sender, &balances_turned_positive));
    }

    Ok(response)
}

pub fn handle_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    // Validate that amount is greater than zero
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidWithdrawalAmount {});
    }

    // Validate that the denom is accepted
    DEPOSIT_ACCEPTED_DENOMS
        .may_load(deps.storage, denom.as_str())?
        .ok_or_else(|| ContractError::DenomNotAccepted {
            denom: denom.clone(),
        })?;

    // Get current balance for this user and denom
    let current_balance = USER_BALANCES
        .may_load(deps.storage, (info.sender.clone(), denom.as_str()))?
        .unwrap_or(0);

    // Validate that user has sufficient balance
    if current_balance < amount.u128() as i128 {
        return Err(ContractError::InsufficientBalance {
            available: current_balance,
            requested: amount,
        });
    }

    // Calculate new balance
    let new_balance = current_balance - amount.u128() as i128;

    // Save the updated balance
    USER_BALANCES.save(
        deps.storage,
        (info.sender.clone(), denom.as_str()),
        &new_balance,
    )?;

    // Create bank message to send tokens to user
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/withdraw")
                .add_attribute("user", info.sender.to_string())
                .add_attribute("denom", denom)
                .add_attribute("amount", amount.to_string())
                .add_attribute("new_balance", new_balance.to_string())
        )
        .add_message(bank_msg))
}

pub fn handle_charge_fees_from_user_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    batch: Vec<UserFees>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    verify_workflow_manager(deps.as_ref(), &info)?;

    let mut users_below_threshold = Vec::new();
    let mut response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/charge_fees_from_user_balance")
                .add_attribute("batch_size", batch.len().to_string())
        );

    // 1. Single iteration - accumulate by (user, denom, fee_type)
    // TODO: This could be done offchain
    let mut user_execution_totals: HashMap<(Addr, String), Uint128> = HashMap::new();
    let mut user_creator_totals: HashMap<(Addr, String), Uint128> = HashMap::new();
    let mut creator_fees_totals: HashMap<(Addr, String), Uint128> = HashMap::new();
    
    for user_fees in &batch {
        for fee in &user_fees.fees {
            // Validate denom
            DEPOSIT_ACCEPTED_DENOMS
                .may_load(deps.storage, fee.denom.as_str())?
                .ok_or_else(|| ContractError::DenomNotAccepted {
                    denom: fee.denom.clone(),
                })?;
            
            match &fee.fee_type {
                FeeType::Execution => {
                    *user_execution_totals
                        .entry((user_fees.user.clone(), fee.denom.clone()))
                        .or_insert(Uint128::zero()) += fee.amount;
                }
                FeeType::Creator { creator_address } => {
                    *user_creator_totals
                        .entry((user_fees.user.clone(), fee.denom.clone()))
                        .or_insert(Uint128::zero()) += fee.amount;
                    *creator_fees_totals
                        .entry((creator_address.clone(), fee.denom.clone()))
                        .or_insert(Uint128::zero()) += fee.amount;
                }
            }
        }
    }

    // 2. Process each user/denom with partial collection logic
    let mut all_users_with_fees: HashSet<(Addr, String)> = HashSet::new();
    all_users_with_fees.extend(user_execution_totals.keys().cloned());
    all_users_with_fees.extend(user_creator_totals.keys().cloned());
    
    for (user, denom) in all_users_with_fees {
        let current_balance = USER_BALANCES
            .may_load(deps.storage, (user.clone(), denom.as_str()))?
            .unwrap_or(0);
        
        let execution_total = user_execution_totals
            .get(&(user.clone(), denom.clone()))
            .unwrap_or(&Uint128::zero())
            .clone();
        let creator_total = user_creator_totals
            .get(&(user.clone(), denom.clone()))
            .unwrap_or(&Uint128::zero())
            .clone();
        
        // Calculate how much can be charged from each type
        let available = current_balance.max(0);
        let execution_chargeable = if available > 0 {
            execution_total.u128().min(available as u128)
        } else {
            0
        };
        let remaining_after_execution = available - execution_chargeable as i128;
        let creator_chargeable = if remaining_after_execution > 0 {
            creator_total.u128().min(remaining_after_execution as u128)
        } else {
            0
        };
        
        // Update balance
        let total_fees_i128 = (execution_total.u128() + creator_total.u128()) as i128;
        let new_balance = current_balance - total_fees_i128;
        USER_BALANCES.save(deps.storage, (user.clone(), denom.as_str()), &new_balance)?;
        
        // Check if balance is below threshold
        if new_balance <= DEPOSIT_ACCEPTED_DENOMS
                .may_load(deps.storage, denom.as_str())?
                .unwrap().min_balance_threshold.u128() as i128 {
            users_below_threshold.push((user.clone(), denom.clone()));
        }
        
        // Update execution_fees only with what could actually be charged
        if execution_chargeable > 0 {
            let current_execution_fees = EXECUTION_FEES
                .may_load(deps.storage, denom.as_str())?
                .unwrap_or(Uint128::zero());
            let new_execution_fees = current_execution_fees + Uint128::from(execution_chargeable);
            EXECUTION_FEES.save(deps.storage, denom.as_str(), &new_execution_fees)?;
        }
        
        // Update creator_fees only with what could actually be charged
        if creator_chargeable > 0 {
            // Find the creator address for this user/denom combination
            let mut found_creator_address = None;
            for user_fees in &batch {
                if user_fees.user == user {
                    for fee in &user_fees.fees {
                        if fee.denom == denom {
                            match &fee.fee_type {
                                FeeType::Creator { creator_address } => {
                                    found_creator_address = Some(creator_address.clone());
                                    break;
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            let creator_address = found_creator_address.unwrap();
            
            let current_creator_fees = CREATOR_FEES
                .may_load(deps.storage, (&creator_address, &denom.as_str()))?
                .unwrap_or(Uint128::zero());
            let new_creator_fees = current_creator_fees + Uint128::from(creator_chargeable);
            CREATOR_FEES.save(deps.storage, (&creator_address, &denom.as_str()), &new_creator_fees)?;
        }
    }

    // 3. Events
    for (user, denom) in users_below_threshold {
        response = response.add_event(balance_below_threshold(&user, &denom));
    }
    Ok(response)
}

pub fn handle_charge_fees_from_message_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fees: Vec<Fee>,
) -> Result<Response, ContractError> {
    // This method is called by the workflow manager but impersonating the user

    // Single iteration: validate fees, accumulate expected funds and creator fees
    // TODO: This validations could be removed as we trust the workflow manager
    let mut expected_funds: HashMap<String, Uint128> = HashMap::new();
    let mut creator_fees_accum: HashMap<(Addr, String), Uint128> = HashMap::new();
    let mut execution_fees_accum: HashMap<String, Uint128> = HashMap::new();
    
    for fee in &fees {
        // Accumulate expected funds
        *expected_funds
            .entry(fee.denom.clone())
            .or_insert(Uint128::zero()) += fee.amount;
        match &fee.fee_type {
            FeeType::Creator { creator_address } => {
                // Accumulate creator fees
                *creator_fees_accum
                    .entry((creator_address.clone(), fee.denom.clone()))
                    .or_insert(Uint128::zero()) += fee.amount;
            }
            FeeType::Execution => {
                // Accumulate execution fees
                *execution_fees_accum
                    .entry(fee.denom.clone())
                    .or_insert(Uint128::zero()) += fee.amount;
            }
        }
    }

    // Validate sent funds match expected funds in single pass
    let mut sent_funds: HashMap<String, Uint128> = HashMap::new();
    for coin in &info.funds {
        *sent_funds.entry(coin.denom.clone()).or_insert(Uint128::zero()) += coin.amount;
    }
    
    // Check all expected funds were sent correctly
    for (denom, expected) in &expected_funds {
        let zero = Uint128::zero();
        let sent = sent_funds.get(denom).unwrap_or(&zero);
        if sent != expected {
            return Err(ContractError::InvalidPayment {
                reason: format!(
                    "Incorrect payment for denom {}: expected {}, got {}",
                    denom, expected, sent
                ),
            });
        }
    }
    
    // Check no unexpected funds were sent
    for (denom, _sent) in &sent_funds {
        if !expected_funds.contains_key(denom) {
            return Err(ContractError::InvalidPayment {
                reason: format!("Unexpected denom sent: {}", denom),
            });
        }
    }

    // Update creator fees storage
    for ((creator, denom), total_fees) in &creator_fees_accum {
        let current = CREATOR_FEES
            .may_load(deps.storage, (creator, denom.as_str()))?
            .unwrap_or(Uint128::zero());
        let new_total = current + *total_fees;
        CREATOR_FEES.save(deps.storage, (creator, denom.as_str()), &new_total)?;
    }

    // Update execution fees storage
    for (denom, total_fees) in &execution_fees_accum {
        let current = EXECUTION_FEES
            .may_load(deps.storage, denom.as_str())?
            .unwrap_or(Uint128::zero());
        let new_total = current + *total_fees;
        EXECUTION_FEES.save(deps.storage, denom.as_str(), &new_total)?;
    }

    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/charge_fees_from_message_coins")
                .add_attribute("fees_count", fees.len().to_string())
        );

    Ok(response)
}

pub fn handle_claim_creator_fees(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let creator = &info.sender;
    let mut total_claimed = Vec::new();
    let mut bank_messages = Vec::new();
    
    // Get all creator fees for this creator using prefix
    let creator_fees_iter = CREATOR_FEES
        .prefix(creator)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some((denom.to_string(), amount))
                } else {
                    None
                }
            })
        });
    
    // Collect all fees and create bank messages
    for (denom, amount) in creator_fees_iter {
        total_claimed.push(Coin {
            denom: denom.clone(),
            amount,
        });
        
        bank_messages.push(BankMsg::Send {
            to_address: creator.to_string(),
            amount: vec![Coin { denom, amount }],
        });
    }
    
    // Check if there are any fees to claim
    if total_claimed.is_empty() {
        return Err(ContractError::NoCreatorFeesToClaim {});
    }
    
    // Clear all creator fees for this creator
    for coin in &total_claimed {
        CREATOR_FEES.remove(deps.storage, (creator, coin.denom.as_str()));
    }
    
    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/claim_creator_fees")
                .add_attribute("creator", creator.to_string())
                .add_attribute("total_claimed", format!("{:?}", total_claimed))
        )
        .add_messages(bank_messages);
    
    Ok(response)
}

pub fn handle_distribute_non_creator_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Verify authorization
    verify_crank(deps.as_ref(), &info)?;

    // Get config to know destination addresses
    let config = CONFIG.load(deps.storage)?;
    
    // Get all execution fees
    let mut total_execution_distributed = Vec::new();
    let mut total_distribution_distributed = Vec::new();
    let mut bank_messages = Vec::new();
    
    // Iterate through all execution fees
    let execution_fees_iter = EXECUTION_FEES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some((denom.to_string(), amount))
                } else {
                    None
                }
            })
        });
    
    // Collect all execution fees and create bank messages
    for (denom, amount) in execution_fees_iter {
        total_execution_distributed.push(Coin {
            denom: denom.clone(),
            amount,
        });
        
        bank_messages.push(BankMsg::Send {
            to_address: config.execution_fees_destination_address.to_string(),
            amount: vec![Coin { denom, amount }],
        });
    }
    
    // Iterate through all distribution fees
    let distribution_fees_iter = DISTRIBUTION_FEES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some((denom.to_string(), amount))
                } else {
                    None
                }
            })
        });
    
    // Collect all distribution fees and create bank messages
    for (denom, amount) in distribution_fees_iter {
        total_distribution_distributed.push(Coin {
            denom: denom.clone(),
            amount,
        });
        
        bank_messages.push(BankMsg::Send {
            to_address: config.distribution_fees_destination_address.to_string(),
            amount: vec![Coin { denom, amount }],
        });
    }
    
    // Check if there are any fees to distribute
    if total_execution_distributed.is_empty() && total_distribution_distributed.is_empty() {
        return Err(ContractError::NoExecutionFeesToDistribute {});
    }
    
    // Clear all execution fees
    for coin in &total_execution_distributed {
        EXECUTION_FEES.remove(deps.storage, coin.denom.as_str());
    }
    
    // Clear all distribution fees
    for coin in &total_distribution_distributed {
        DISTRIBUTION_FEES.remove(deps.storage, coin.denom.as_str());
    }
    
    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/distribute_non_creator_fees")
                .add_attribute("execution_destination", config.execution_fees_destination_address.to_string())
                .add_attribute("distribution_destination", config.distribution_fees_destination_address.to_string())
                .add_attribute("execution_distributed", format!("{:?}", total_execution_distributed))
                .add_attribute("distribution_distributed", format!("{:?}", total_distribution_distributed))
        )
        .add_messages(bank_messages);
    
    Ok(response)
}

pub fn handle_distribute_creator_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Verify authorization
    verify_crank(deps.as_ref(), &info)?;

    // Get config to know destination address and distribution fee
    let config = CONFIG.load(deps.storage)?;
    
    // Get all subscribed creators first
    let subscribed_creators: Vec<Addr> = SUBSCRIBED_CREATORS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(creator, is_subscribed)| {
                if is_subscribed {
                    Some(creator)
                } else {
                    None
                }
            })
        })
        .collect();
    
    // If no creators are subscribed, return error
    if subscribed_creators.is_empty() {
        return Err(ContractError::NoCreatorFeesToDistribute {});
    }
    
    let mut total_distributed = Vec::new();
    let mut bank_messages = Vec::new();
    let mut distribution_fees_accum = std::collections::HashMap::new();
    let mut creators_to_remove = Vec::new();
    
    // Process fees only for subscribed creators
    for creator in subscribed_creators {
        // Get all fees for this creator
        let creator_fees: Vec<(String, Uint128)> = CREATOR_FEES
            .prefix(&creator)
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .filter_map(|result| {
                result.ok().and_then(|(denom, amount)| {
                    if amount > Uint128::zero() {
                        Some((denom.to_string(), amount))
                    } else {
                        None
                    }
                })
            })
            .collect();
        
        // Process each denom for this creator
        for (denom, amount) in creator_fees {
            // Calculate distribution fee (creator_distribution_fee is in basis points, e.g., 5 = 0.05%)
            let distribution_fee = amount
                .checked_mul(config.creator_distribution_fee)
                .map_err(|_| ContractError::Std(cosmwasm_std::StdError::generic_err("Overflow")))?
                .checked_div(Uint128::from(100u128))
                .map_err(|_| ContractError::Std(cosmwasm_std::StdError::generic_err("Divide by zero")))?;
            
            // Calculate amount to send to creator (total - distribution fee)
            let amount_to_creator = amount
                .checked_sub(distribution_fee)
                .map_err(|_| ContractError::Std(cosmwasm_std::StdError::generic_err("Overflow")))?;
            
            // Accumulate distribution fees
            *distribution_fees_accum.entry(denom.clone()).or_insert(Uint128::zero()) += distribution_fee;
            
            // Create bank message to send to creator
            if amount_to_creator > Uint128::zero() {
                total_distributed.push(Coin {
                    denom: denom.clone(),
                    amount: amount_to_creator,
                });
                
                bank_messages.push(BankMsg::Send {
                    to_address: creator.to_string(),
                    amount: vec![Coin { denom: denom.clone(), amount: amount_to_creator }],
                });
                
                // Mark this creator's fees for removal
                creators_to_remove.push((creator.clone(), denom));
            }
        }
    }
    
    // Check if there are any creator fees to distribute
    if total_distributed.is_empty() {
        return Err(ContractError::NoCreatorFeesToDistribute {});
    }
    
    // Update DISTRIBUTION_FEES storage
    for (denom, total_fees) in &distribution_fees_accum {
        let current = DISTRIBUTION_FEES
            .may_load(deps.storage, denom.as_str())?
            .unwrap_or(Uint128::zero());
        let new_total = current + *total_fees;
        DISTRIBUTION_FEES.save(deps.storage, denom.as_str(), &new_total)?;
    }
    
    // Clear only the creator fees that were distributed
    for (creator, denom) in creators_to_remove {
        CREATOR_FEES.remove(deps.storage, (&creator, denom.as_str()));
    }
    
    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/distribute_creator_fees")
                .add_attribute("distribution_fee_rate", config.creator_distribution_fee.to_string())
                .add_attribute("total_distributed", format!("{:?}", total_distributed))
        )
        .add_messages(bank_messages);
    
    Ok(response)
}

pub fn has_exceeded_debt_limit(deps: Deps, user: Addr) -> StdResult<bool> {
    let accepted_denoms: Vec<(String, AcceptedDenomValue)> = DEPOSIT_ACCEPTED_DENOMS
    .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
    .map(|item| {
        let (key, value) = item.unwrap();
        (key.to_string(), value)
    })
    .collect();

    for accepted_denom in accepted_denoms {
        let user_balance = USER_BALANCES
            .may_load(deps.storage, (user.clone(), accepted_denom.0.as_str()))?
            .unwrap_or(0);

        // If the balance is negative (debt), check if it exceeds the max_debt amount
        if user_balance < 0 {
            // Convert the negative balance to a positive amount for comparison
            let debt_amount = (-user_balance) as u128;
            // Check if the debt exceeds the max_debt limit
            if debt_amount > accepted_denom.1.max_debt.u128() {
                return Ok(true);
            }
        }
    }
    Ok(false)    
}

pub fn get_user_balances(deps: Deps, user: Addr) -> StdResult<crate::msg::UserBalancesResponse> {
    // Get accepted denoms to know which balances to check
    let accepted_denoms: Vec<String> = DEPOSIT_ACCEPTED_DENOMS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (key, _) = item.unwrap();
            key.to_string()
        })
        .collect();

    let mut balances = Vec::new();
    
    // Get balance for each accepted denom
    for accepted_denom in accepted_denoms {
        let balance = USER_BALANCES
            .may_load(deps.storage, (user.clone(), accepted_denom.as_str()))?
            .unwrap_or(0);
        
        balances.push(crate::msg::UserBalance {
            denom: accepted_denom,
            balance,
        });
    }
    
    Ok(crate::msg::UserBalancesResponse {
        user,
        balances,
    })
}

pub fn get_creator_fees(deps: Deps, creator: Addr) -> StdResult<crate::msg::CreatorFeesResponse> {
    // Get all creator fees for this creator using prefix
    let creator_fees_iter = CREATOR_FEES
        .prefix(&creator)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some((denom.to_string(), amount))
                } else {
                    None
                }
            })
        });

    let mut fees = Vec::new();

    for (denom, balance) in creator_fees_iter {
        fees.push(crate::msg::FeeBalance {
            denom: denom,
            balance: balance,
        });
    }
    
    Ok(crate::msg::CreatorFeesResponse {
        creator,
        fees,
    })
}

pub fn get_non_creator_fees(deps: Deps) -> StdResult<crate::msg::NonCreatorFeesResponse> {
    let execution_fees = EXECUTION_FEES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some(crate::msg::FeeBalance {
                        denom: denom.to_string(),
                        balance: amount,
                    })
                } else {
                    None
                }
            })
        })
        .collect();
    
    let distribution_fees = DISTRIBUTION_FEES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(denom, amount)| {
                if amount > Uint128::zero() {
                    Some(crate::msg::FeeBalance {
                        denom: denom.to_string(),
                        balance: amount,
                    })
                } else {
                    None
                }
            })
        })
        .collect();
    
    Ok(crate::msg::NonCreatorFeesResponse { execution_fees, distribution_fees })
}

pub fn handle_enable_creator_fee_distribution(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Only the creator can enable their own fee distribution
    let creator = info.sender;
    
    // Set the creator as subscribed
    SUBSCRIBED_CREATORS.save(deps.storage, &creator, &true)?;
    
    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/enable_creator_fee_distribution")
                .add_attribute("creator", creator.to_string())
        );
    
    Ok(response)
}

pub fn handle_disable_creator_fee_distribution(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Only the creator can disable their own fee distribution
    let creator = info.sender;
    
    // Set the creator as unsubscribed
    SUBSCRIBED_CREATORS.save(deps.storage, &creator, &false)?;
    
    let response = Response::new()
        .add_event(
            cosmwasm_std::Event::new("autorujira-fee-manager/disable_creator_fee_distribution")
                .add_attribute("creator", creator.to_string())
        );
    
    Ok(response)
}

pub fn is_creator_subscribed(deps: Deps, creator: Addr) -> StdResult<bool> {
    // Check if the creator is subscribed to fee distribution
    let is_subscribed = SUBSCRIBED_CREATORS
        .may_load(deps.storage, &creator)?
        .unwrap_or(false);
    
    Ok(is_subscribed)
}

pub fn get_subscribed_creators(deps: Deps) -> StdResult<crate::msg::SubscribedCreatorsResponse> {
    let mut creators = Vec::new();
    
    // Iterate through all subscribed creators
    let subscribed_creators_iter = SUBSCRIBED_CREATORS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|result| {
            result.ok().and_then(|(creator, is_subscribed)| {
                if is_subscribed {
                    Some(creator)
                } else {
                    None
                }
            })
        });
    
    for creator in subscribed_creators_iter {
        creators.push(creator);
    }
    
    Ok(crate::msg::SubscribedCreatorsResponse { creators })
}
