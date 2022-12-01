use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

use group_admin::GroupAdminError;
use saleable::error::SaleableItemError;

#[derive(Error, Debug, PartialEq)]
pub enum ManagedServiceError {
    #[error("{0}")]
    Std(#[from] StdError),


    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    SaleItemErrors (#[from] SaleableItemError),

    #[error("SaleServiceNotEnabled")]
    SaleServiceNotEnabled {},

    #[error("{0}")]
    GroupAdminHooksError(#[from] GroupAdminError),

    #[error("AddressFormatError. Given contract address is not correctly formatted")]
    ManagerContractAddressFormatError { inner_error: StdError},

    #[error("NoManagerContractAddressProvided")]
    NoManagerContractAddressProvided {},
}


/*

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    GroupAdminHooksError(#[from] GroupAdminError),

    #[error("{0}")]
    SaleItemErrors (#[from] SaleableItemError),

    #[error("{0}")]
    ManagedServiceError(#[from] ManagedServiceError),
 */
