use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("UnknownPositionType")]
    UnknownPositionType {},

    #[error("PlayerNameAlreadyInUse")]
    PlayerNameAlreadyInUse {first_name: String, last_name: String},

    #[error("PlayerNameAlreadyInUse")]
    PlayerNameCheckError {error: StdError},

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
