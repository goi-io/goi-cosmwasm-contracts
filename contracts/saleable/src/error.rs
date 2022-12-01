use cosmwasm_std::{StdError, Uint128};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SaleableItemError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("PriceNotSet")]
    PriceNotSet {},

    #[error("InvalidPrice")]
    InvalidPrice {},


    #[error("NotForSale")]
    NotForSale {},


    #[error("EmptyBalance")]
    EmptyBalance {},

    #[error("SaleableItemFailure")]
    SaleableItemFailure {},


    #[error("CurrentOwnersRequired")]
    CurrentOwnersRequired {},


    #[error("Price provided is not current")]
    PriceNotCurrentError {
        denom_current: String,
        denom_provided: String,
        price_current: Uint128,
        price_provided: Uint128,
    },

    #[error("IncorretFunds")]
    IncorretFunds {},


    #[error("Insufficient funds sent")]
    InsufficientFundsSend {},



    #[error("OwnershipTransferFailure")]
    OwnershipTransferFailure {},



    #[error("Unauthorized")]
    Unauthorized {},



    #[error("Total ownership of all members has to equal 100%")]
    OwnershipRequirementNotMet {},


    #[error("FailureTransferingOwnership")]
    FailureTransferingOwnership {},



    #[error("NotAnAdmin")]
    NotAnAdmin {},


    #[error("AdminError")]
    AdminError { admin_error: AdminError },


}


impl From<AdminError> for SaleableItemError {
    fn from(err: AdminError) -> Self {
        SaleableItemError::AdminError{ admin_error: err }
    }

}
