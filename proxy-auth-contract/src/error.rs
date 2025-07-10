use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Invalid action type")]
    InvalidActionType {},

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("Bid {bid_id} not found")]
    BidNotFound {
        bid_id: String
	},

    #[error("Bid {bid_id} has no PostBid with id {post_bid_id}")]
    PostBidNotFound {
        bid_id: String,
        post_bid_id: String,
	},

    #[error("PostBid {post_bid_id} (Bid {bid_id}) hasn't Inactive state, can't activate PostBid) ")]
    PostBidActivationStateError {
        bid_id: String,
        post_bid_id: String,
	},

    #[error("can't execute post bids because bid {bid_id} has been created with config.post_action=None")]
	PostBidActivationBidStateError {
        bid_id: String,
	},

    #[error("Bid has no balance or the amount specified is greater than the available balance")]
    BidHasNoBalance {},

    #[error("Don't send tokens to this function")]
    InvalidFundsReceived {},

    #[error("attribute wasm.bid_idx not found in reply")]
    OrcaBidIdxNotFoundInReply {},

    #[error("Invocation error: {0}")]
    InvocationError(String),

    #[error("address {0} has no bids")]
    NoBidsForAddress(String),

    #[error("a max of {0} bids are supported")]
    TooManyBids(String),

    #[error("bid {bid_idx} from contract {contract_addr} has only {balance} in balance")]
    InsufficientAmount {
        bid_idx: String,
        contract_addr: String,
        balance: String,
    },

    #[error("{0}")]
	GenericError(String) ,

    #[error("No funds to reclaim")]
    NoFundsToClaim {},
    #[error("Error transferring funds.")]
    TransferError {},

    #[error("Template {template_id} is already approved")]
    TemplateAlreadyApproved {
        template_id: String,
    },

    #[error("Template {template_id} already exists")]
    TemplateAlreadyExists {
        template_id: String,
    },
}
