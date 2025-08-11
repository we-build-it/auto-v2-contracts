use cosmwasm_std::{Addr, Event};

/// Event emitted when a user's balance turns from negative to positive during deposit
pub fn deposit_completed(user: &Addr, balances_turned_positive: &[String]) -> Event {
    Event::new("deposit_completed")
        .add_attribute("user", user.to_string())
        .add_attribute("balances_turned_positive", balances_turned_positive.join(","))
}

/// Event emitted when a user's balance falls below the minimum threshold
pub fn balance_below_threshold(user: &Addr, denom: &str) -> Event {
    Event::new("balance_below_threshold")
        .add_attribute("user", user.to_string())
        .add_attribute("denom", denom)
}
