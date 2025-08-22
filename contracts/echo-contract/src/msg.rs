use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Echo any message and emit an event
    Echo {
        message: Binary,
    },
    // Echo with custom event attributes
    EchoWithAttributes {
        message: Binary,
        attributes: Vec<(String, String)>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(MessageCountResponse)]
    MessageCount {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
}

#[cw_serde]
pub struct MessageCountResponse {
    pub count: u64,
}
