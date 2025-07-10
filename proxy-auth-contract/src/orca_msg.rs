use cosmwasm_std::{Decimal256, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrcaExecuteMsg {
    SubmitBid { premium_slot: u16 },
    RetractBid { bid_idx: String, amount: Option<Uint128> },
    ActivateBids { bids_idx: Option<Vec<Uint128>> },
    ClaimLiquidations { bids_idx: Vec<Uint128> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FinExecuteMsg {
    SubmitOrder { price: Decimal256 },
	WithdrawOrders {
        order_idxs: Vec<Uint128>,
    },
	RetractOrders {
        order_idxs: Vec<Uint128>,
    },
}

