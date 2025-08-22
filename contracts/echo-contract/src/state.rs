use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const MESSAGE_COUNT: Item<u64> = Item::new("message_count");

pub fn get_message_count(storage: &dyn Storage) -> u64 {
    MESSAGE_COUNT.may_load(storage).unwrap_or(Some(0)).unwrap_or(0)
}

pub fn increment_message_count(storage: &mut dyn Storage) -> u64 {
    let current_count = get_message_count(storage);
    let new_count = current_count + 1;
    MESSAGE_COUNT.save(storage, &new_count).unwrap();
    new_count
}
