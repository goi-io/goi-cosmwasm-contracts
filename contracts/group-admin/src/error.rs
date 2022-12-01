use cosmwasm_std::StdError;
use cw_controllers::{AdminError, HookError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum GroupAdminError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    HookError(#[from] HookError),



    #[error("{0}")]
    GroupError(#[from] cw4_group::error::ContractError),


    #[error("{0}")]
    AdminError(#[from] AdminError),



    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("Unauthorized")]
    OwnershipMembersRequirementNotMet {message: String}
}
