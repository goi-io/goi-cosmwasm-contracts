use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ManagementError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("RemoveItemsNotFound")]
    RemoveItemsNotFound {},


    #[error("ContractNotFound")]
    ContractNotFound {},


    #[error("ContractAlreadyAdded")]
    ContractAlreadyAdded {}

}



