use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub max_debt: Coin,
    pub min_balance_threshold: Coin,
    pub execution_fees_destination_address: Addr,
    pub distribution_fees_destination_address: Addr,
    pub crank_authorized_address: Addr,
    pub workflow_manager_address: Addr,
    pub creator_distribution_fee: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

// user address → denom → balance (can be negative)
pub const USER_BALANCES: Map<(Addr, &str), i128> = Map::new("user_balances");

// user address → denom → creator fee balance
pub const CREATOR_FEES: Map<(&Addr, &str), Uint128> = Map::new("creator_fees");

// denom → execution fee balance
pub const EXECUTION_FEES: Map<&str, Uint128> = Map::new("execution_fees");

// denom → distribution fee balance
pub const DISTRIBUTION_FEES: Map<&str, Uint128> = Map::new("distribution_fees");

// Defines which tokens are accepted for deposits
pub const ACCEPTED_DENOMS: Item<Vec<String>> = Item::new("accepted_denoms");

// creator address → subscription status for fee distribution
pub const SUBSCRIBED_CREATORS: Map<&Addr, bool> = Map::new("subscribed_creators");
