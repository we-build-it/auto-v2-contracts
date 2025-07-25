use crate::events::{balance_below_threshold, deposit_completed};
use crate::helpers::{verify_authorization, verify_workflow_manager};
use crate::msg::{Fee, FeeType};
use crate::state::{ACCEPTED_DENOMS, CONFIG, CREATOR_FEES, EXECUTION_FEES, USER_BALANCES};
use crate::{error::ContractError, msg::UserFees};
use cosmwasm_std::{
    Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use hashbrown::HashMap;

pub fn handle_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Validate that funds were sent
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    // Load accepted denoms
    let accepted_denoms = ACCEPTED_DENOMS.load(deps.storage)?;

    // Track denoms that turned positive
    let mut balances_turned_positive = Vec::new();

    // Process each coin sent
    for coin in &info.funds {
        // Validate that the denom is accepted
        if !accepted_denoms.contains(&coin.denom) {
            return Err(ContractError::DenomNotAccepted {
                denom: coin.denom.clone(),
            });
        }

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
        .add_attribute("method", "deposit")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("funds", format!("{:?}", info.funds));

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
    // Validate that amount is greater than zero
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidWithdrawalAmount {});
    }

    // Load accepted denoms
    let accepted_denoms = ACCEPTED_DENOMS.load(deps.storage)?;

    // Validate that the denom is accepted
    if !accepted_denoms.contains(&denom) {
        return Err(ContractError::DenomNotAccepted {
            denom: denom.clone(),
        });
    }

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
        .add_message(bank_msg)
        .add_attribute("method", "withdraw")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("denom", denom)
        .add_attribute("amount", amount.to_string())
        .add_attribute("new_balance", new_balance.to_string()))
}

pub fn handle_charge_fees_from_user_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    batch: Vec<UserFees>,
) -> Result<Response, ContractError> {
    verify_authorization(deps.as_ref(), &info)?;
    let config = CONFIG.load(deps.storage)?;
    let accepted_denoms = ACCEPTED_DENOMS.load(deps.storage)?;

    let mut users_below_threshold = Vec::new();
    let mut response = Response::new()
        .add_attribute("method", "charge_fees_from_user_balance")
        .add_attribute("batch_size", batch.len().to_string());

    // Temporary accumulators
    let mut user_balances_accum: HashMap<(Addr, String), i128> = HashMap::new();
    let mut execution_fees_accum: HashMap<String, Uint128> = HashMap::new();
    let mut creator_fees_accum: HashMap<(Addr, String), Uint128> = HashMap::new();

    // 1. Accumulate in memory
    // TODO: This could be done offchain
    for user_fees in &batch {
        for fee in &user_fees.fees {
            // Validate denom
            if !accepted_denoms.contains(&fee.denom) {
                return Err(ContractError::DenomNotAccepted {
                    denom: fee.denom.clone(),
                });
            }
            // Accumulate balance and fees with available balance logic
            let key = (user_fees.user.clone(), fee.denom.clone());
            let user_balance = user_balances_accum.entry(key.clone()).or_insert(0);
            // Calculate the current balance before this fee
            let current_balance = USER_BALANCES
                .may_load(deps.storage, (user_fees.user.clone(), fee.denom.as_str()))?
                .unwrap_or(0)
                + *user_balance;
            let fee_to_charge = fee.amount.u128() as i128;
            let chargeable = if current_balance > 0 {
                fee_to_charge.min(current_balance)
            } else {
                0
            };
            // Always subtract the full fee from the user's balance
            *user_balance -= fee_to_charge;
            // Only add what could actually be charged to the fee accumulators
            match fee.fee_type {
                FeeType::Execution => {
                    *execution_fees_accum
                        .entry(fee.denom.clone())
                        .or_insert(Uint128::zero()) += Uint128::from(chargeable as u128);
                }
                FeeType::Creator => {
                    let creator_addr = fee.creator_address.as_ref().ok_or_else(|| {
                        ContractError::InvalidCreatorAddress {
                            reason: "creator_address is required for Creator fees".to_string(),
                        }
                    })?;
                    *creator_fees_accum
                        .entry((creator_addr.clone(), fee.denom.clone()))
                        .or_insert(Uint128::zero()) += Uint128::from(chargeable as u128);
                }
            }
        }
    }

    // 2. Load current balances and add accumulated deltas
    for ((user, denom), delta) in &user_balances_accum {
        let current = USER_BALANCES
            .may_load(deps.storage, (user.clone(), denom.as_str()))?
            .unwrap_or(0);
        let new_balance = current + delta;
        USER_BALANCES.save(deps.storage, (user.clone(), denom.as_str()), &new_balance)?;
        if new_balance <= config.min_balance_threshold.amount.u128() as i128 {
            users_below_threshold.push((user.clone(), denom.clone()));
        }
    }
    for (denom, delta) in &execution_fees_accum {
        let current = EXECUTION_FEES
            .may_load(deps.storage, denom.as_str())?
            .unwrap_or(Uint128::zero());
        let new_total = current + *delta;
        EXECUTION_FEES.save(deps.storage, denom.as_str(), &new_total)?;
    }
    for ((creator, denom), delta) in &creator_fees_accum {
        let current = CREATOR_FEES
            .may_load(deps.storage, (creator, denom.as_str()))?
            .unwrap_or(Uint128::zero());
        let new_total = current + *delta;
        CREATOR_FEES.save(deps.storage, (creator, denom.as_str()), &new_total)?;
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
    _fees: Vec<Fee>,
    _creator_address: Addr,
) -> Result<Response, ContractError> {
    // Verify workflow manager authorization
    verify_workflow_manager(deps.as_ref(), &info)?;

    // TODO: Implement onchain fees charging logic
    Ok(Response::new())
}

pub fn handle_claim_creator_fees(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    // TODO: Implement creator fees claiming logic
    Ok(Response::new())
}

pub fn handle_distribute_non_creator_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Verify authorization
    verify_authorization(deps.as_ref(), &info)?;

    // TODO: Implement non-creator fees distribution logic
    Ok(Response::new())
}

pub fn handle_distribute_creator_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Verify authorization
    verify_authorization(deps.as_ref(), &info)?;

    // TODO: Implement creator fees distribution logic
    Ok(Response::new())
}

pub fn has_exceeded_debt_limit(_deps: Deps, _user: Addr) -> StdResult<bool> {
    // TODO: Implement debt limit checking logic
    Ok(false)
}
