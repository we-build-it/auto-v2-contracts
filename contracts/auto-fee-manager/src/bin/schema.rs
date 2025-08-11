use cosmwasm_schema::write_api;

use auto_fee_manager::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        migrate: MigrateMsg,
    }
}
