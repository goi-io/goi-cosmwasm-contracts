use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use cw_utils::ParseReplyError;
use thiserror::Error;

use group_admin::GroupAdminError;
use managed::ManagedServiceError;
use saleable::error::SaleableItemError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    GroupAdminHooksError(#[from] GroupAdminError),

    #[error("{0}")]
    SaleItemErrors (#[from] SaleableItemError),

    #[error("{0}")]
    ManagedServiceError(#[from] ManagedServiceError),

    #[error("{0}")]
    ParseReplyDataError(#[from]  ParseReplyError),

    #[error("TaskContractCodeIdNotSet")]
    TaskContractCodeIdNotSet {},


    #[error("InvalidReplyId")]
    InvalidReplyId { message: String},

    #[error("ReplyProcessingFailed")]
    ReplyProcessingFailed {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotFound")]
    TaskNotFound {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.



}
