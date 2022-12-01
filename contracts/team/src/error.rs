use cosmwasm_std::{Addr, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

use group_admin::GroupAdminError;
use managed::error::ManagedServiceError;
use shared::{player::InstantiateMsg, player_attributes::Positions};
use saleable::error::SaleableItemError;

#[derive(Error, Debug, PartialEq)]
pub enum TeamError {
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
    Unauthorized {},

    #[error("UnauthorizedSender")]
    UnauthorizedSender { sender: Addr},

    #[error("UnknownError")]
    UnknownError {},

    #[error("PositionAlreadyAssigned")]
    PositionAlreadyAssigned {},

    #[error("PositionNotAssigned")]
    PositionNotAssigned {position: Positions},

    #[error("ContractNotFound")]
    ContractNotFound {},


    #[error("PositionNotAssigned")]
    RequestedPlayerPositionConflict {player_address: Addr, position: Positions},

    #[error("PlayerPositionAssignmentConflicts")]
    PlayerPositionAssignmentConflicts {},

    #[error("IllegalPositionAssignment")]
    IllegalPositionAssignment { },

    #[error("PlayerDeclaredPosAndAssignPosMisMatch")]
    PlayerDeclaredPosAndAssignPosMisMatch { player_address: Addr,
                                            position: Positions,
                                            contract_first_name: String,
                                            contract_last_name: String,
                                            contract_position: Positions},

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


    #[error("PositionAssignmentsNotProvided")]
    PositionAssignmentsNotProvided {},

    #[error("GenericErr")]
    GenericErr { message: String },

    #[error("ErrorCreatingPlayer")]
    ErrorCreatingPlayer { msg: InstantiateMsg},


}




