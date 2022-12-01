use cosmwasm_std::{Addr, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

use group_admin::GroupAdminError;
use managed::error::ManagedServiceError;
use shared::{player::InstantiateMsg, player_attributes::Positions};
use saleable::error::SaleableItemError;

#[derive(Error, Debug, PartialEq)]
pub enum LeagueError {
    #[error("{0}")]
    Std(#[from] StdError),


    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    GroupAdminHooksError(#[from] GroupAdminError),


    #[error("{0}")]
    SaleItemErrors (#[from] SaleableItemError),

    #[error("{0}")]
    ManagableServiceError(#[from] ManagedServiceError),



    #[error("Unauthorized")]
    Unauthorized { sender: Addr},

    #[error("UnknownError")]
    UnknownError {},



    #[error("ContractNotFound")]
    ContractNotFound {},



    #[error("UpdateMembershipFailure")]
    UpdateMembershipFailure {},


    #[error("Owner count requirement not met. Minimum of 1; maximum owners is 10")]
    OwnershipMembersRequirementNotMet { message: String},

    #[error("DivideByZeroError")]
    DivideByZeroError {},

    #[error("SubtractionError")]
    SubtractionError {},


    #[error("StalePriceVersion")]
    StalePriceVersion { message: String},


    #[error("ItemNotFound")]
    ItemNotFound {},

    #[error("FailedToInitializeContract")]
    FailedToInitializeContract {},



    #[error("GenericErr")]
    GenericErr { message: String },



}




