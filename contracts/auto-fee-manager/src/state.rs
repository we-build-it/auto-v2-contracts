use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::AcceptedDenomValue;

#[cw_serde]
pub struct Config {
    pub execution_fees_destination_address: Addr,
    pub distribution_fees_destination_address: Addr,
    pub crank_authorized_address: Addr,
    pub workflow_manager_address: Option<Addr>,
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
pub const ACCEPTED_DENOMS: Map<&str, AcceptedDenomValue> = Map::new("accepted_denoms_new");

// creator address → subscription status for fee distribution
pub const SUBSCRIBED_CREATORS: Map<&Addr, bool> = Map::new("subscribed_creators");
