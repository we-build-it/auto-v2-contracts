use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    // Maximum debt that can be incurred by a user
    pub max_debt: Coin,
    // Minimum balance threshold for triggering events
    pub min_balance_threshold: Coin,
    // Address that will receive gas fees
    pub gas_destination_address: Addr,
    // Address that will receive infra fees
    pub infra_destination_address: Addr,
    // Denoms that are accepted for deposits
    pub accepted_denoms: Vec<String>,
    // Address that is authorized to charge fees from the crank contract
    pub crank_authorized_address: Addr,
    // Address of the workflow manager contract
    pub workflow_manager_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Withdraw {
        denom: String,
        amount: Uint128,
    },
    ChargeFeesFromUserBalance {
        batch: Vec<UserFees>,
    },
    ChargeFeesFromMessageCoins {
        fees: Vec<Fee>,
        creator_address: Addr,
    },
    ClaimCreatorFees {},
    DistributeCreatorFees {},
    DistributeNonCreatorFees {},
}

#[cw_serde]
pub enum SudoMsg {
    SetMaxDebt { denom: String, amount: Decimal },
    SetReceiverAddress { fee_type: FeeType, address: Addr },
    SetCrankAuthorizedAddress { address: Addr },
    SetWorkflowManagerAddress { address: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    HasExceededDebtLimit { user: Addr },
}

#[cw_serde]
pub struct UserFees {
    pub user: Addr,
    pub fees: Vec<Fee>,
}

#[cw_serde]
pub struct Fee {
    pub workflow_instance_id: String,
    pub action_id: String,
    pub description: String,
    pub timestamp: u64,
    pub amount: Uint128,
    pub denom: String,
    pub fee_type: FeeType,
    pub creator_address: Option<Addr>, // Only populated when fee_type = Creator
}

#[cw_serde]
pub enum FeeType {
    Execution,
    Creator,
}
