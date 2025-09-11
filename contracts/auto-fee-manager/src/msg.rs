use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    // Denoms that are accepted for deposits
    pub accepted_denoms: Vec<AcceptedDenom>,

    // Address that will receive execution fees
    pub execution_fees_destination_address: Addr,
    // Address that will receive distribution fees
    pub distribution_fees_destination_address: Addr,
    // Address that is authorized to charge fees from the crank contract
    pub crank_authorized_address: Addr,
    // Address of the workflow manager contract
    pub workflow_manager_address: Option<Addr>,
    // Creator distribution fee (e.g., 0.05 for 5%)
    pub creator_distribution_fee: Uint128,
}

#[cw_serde]
pub struct AcceptedDenom {
    pub denom: String,
    // Maximum debt that can be incurred by a user
    pub max_debt: Uint128,
    // Minimum balance threshold for triggering events
    pub min_balance_threshold: Uint128,
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
    },
    ClaimCreatorFees {},
    DistributeCreatorFees {},
    DistributeNonCreatorFees {},
    EnableCreatorFeeDistribution {},
    DisableCreatorFeeDistribution {},
}

#[cw_serde]
pub enum SudoMsg {
    SetCrankAuthorizedAddress { address: Addr },
    SetWorkflowManagerAddress { address: Addr },
    SetExecutionFeesDestinationAddress { address: Addr },
    SetDistributionFeesDestinationAddress { address: Addr },
    SetCreatorDistributionFee { fee: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    HasExceededDebtLimit { user: Addr },
    #[returns(UserBalancesResponse)]
    GetUserBalances { user: Addr },
    #[returns(CreatorFeesResponse)]
    GetCreatorFees { creator: Addr },
    #[returns(NonCreatorFeesResponse)]
    GetNonCreatorFees {},
    #[returns(bool)]
    IsCreatorSubscribed { creator: Addr },
    #[returns(SubscribedCreatorsResponse)]
    GetSubscribedCreators {},
    #[returns(InstantiateMsg)]
    GetConfig {},
}

#[cw_serde]
pub struct UserBalancesResponse {
    pub user: Addr,
    pub balances: Vec<UserBalance>,
}

#[cw_serde]
pub struct UserBalance {
    pub denom: String,
    pub balance: i128,
}

// ChargeFeesFromUserBalance has a vector of UserFees
#[cw_serde]
pub struct UserFees {
    pub user: Addr,
    pub fees: Vec<Fee>,
}

// ChargeFeesFromMessageCoins has a vector of Fee
#[cw_serde]
pub struct Fee {
    pub fee_type: FeeType,
    pub denom: String,
    pub amount: Uint128,
}

#[cw_serde]
pub enum FeeType {
    Execution,
    Creator { creator_address: Addr },
}

#[cw_serde]
pub struct MigrateMsg {
    // Empty for now, can be extended in future migrations
}

#[cw_serde]
pub struct CreatorFeesResponse {
    pub creator: Addr,
    pub fees: Vec<FeeBalance>,
}

#[cw_serde]
pub struct NonCreatorFeesResponse {
    pub execution_fees: Vec<FeeBalance>,
    pub distribution_fees: Vec<FeeBalance>,
}

#[cw_serde]
pub struct FeeBalance {
    pub denom: String,
    pub balance: Uint128,
}

#[cw_serde]
pub struct SubscribedCreatorsResponse {
    pub creators: Vec<Addr>,
}
