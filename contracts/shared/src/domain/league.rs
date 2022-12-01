use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Timestamp, WasmMsg, to_binary, CosmosMsg, SubMsg, ReplyOn};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::goi_manager;
use crate::manage::ManagedStatus;
use crate::season::Season;
use crate::utils::{BlockTime, IManaged, LeagueAddr, StartDate, TeamAddr};
use crate::utils::general::AssetTypes;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LeagueTypes {
    NotSet,
    Human,
    AI,
    Mixed
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SeasonActiveStatus {
    NotSet,
    Completed,
    InProgress,
    Upcoming,
    Cancelled,
}

impl SeasonActiveStatus {
    pub fn get_u8(&self) -> u8 {
        match &self {
            SeasonActiveStatus::NotSet => {
                0u8
            },
            SeasonActiveStatus::Completed => {
                1u8
            },
            SeasonActiveStatus::InProgress => {
                2u8
            },
            SeasonActiveStatus::Upcoming => {
                3u8
            },
            SeasonActiveStatus::Cancelled => {
                4u8
            }

        }
    }
}


impl Default for LeagueTypes {
    fn default() -> Self {
        LeagueTypes::NotSet
    }
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeagueTeamLink {
    pub id: u64,
    pub league_addr: LeagueAddr,
    pub team_addr: TeamAddr,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeagueInfo {
    pub address: Addr,
    pub owner: Addr,
    pub name: String,
    pub league_type: LeagueTypes,
    pub created: BlockTime,

}

impl LeagueInfo {
    // Constructs a new instance of [`Second`].
    // Note this is an associated function - no self.
    pub fn new(name: String, addr: Addr, owner_addr: Addr, block_time: BlockTime) -> Self {
        Self{
            address: addr,
            owner: owner_addr,
            name,
            league_type: Default::default(),
            created: BlockTime{
                height: block_time.height,
                time: block_time.time,
                chain_id: block_time.chain_id
            },

        }

    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeagueTeamAssignment {
    pub league: LeagueAddr,
    pub assigned_date: Timestamp,

}










#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetInfo{},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub info: Season,
}



#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},


    #[error("StartTimeEndTimeViolation")]
    StartTimeEndTimeViolation {},

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}


pub fn set_start_and_end_date
(deps: DepsMut, _env: Env, info: MessageInfo,
 start_date: Timestamp, end_date: Timestamp, state: &Item<Season>)
 -> Result<Response, ContractError>
{
    let mut current_state = state.load(deps.storage)?;
    match current_state.league == info.sender {
        true => {
            match validate_start_and_end_time
                (Some(start_date), Some(end_date), _env) {
                true => {
                    current_state.start_date = start_date;
                    current_state.end_date = end_date;
                    state.save(deps.storage, &current_state).expect("Failed to update start and end dates.");
                    Ok(Response::default())
                }
                false => Err(ContractError::StartTimeEndTimeViolation {})
            }
        }
        false => Err(ContractError::Unauthorized {})
    }


}


pub fn validate_start_and_end_time(new_start: Option<Timestamp>, new_end: Option<Timestamp>, _env: Env) -> bool {
    match (new_start, new_end) {
        (Some(start), Some(end)) =>  if start < _env.block.time ||
            end < start { false } else { true },
        _ => false
    }
}


