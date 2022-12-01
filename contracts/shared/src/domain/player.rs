use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{player_attributes::{Positions, PlayerAttributes}, utils::general::GameItemTypes};
use crate::utils::PlayerAddr;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerTrackingItem {
    pub game_piece_type: GameItemTypes,
    pub player_info: PlayerInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerInfoPacket {
    pub items: Vec<PlayerInfo>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerInfo {
    pub address: PlayerAddr,
    pub first_name: String,
    pub last_name: String,
    pub position: Positions,
    pub assigned_team_address: Option<Addr>,
}

impl PlayerInfo {
    pub fn new (player_addr: PlayerAddr, first_name: String, last_name: String, position: Positions, assigned_team_address: Option<Addr>) -> Self {
        Self {
            address: player_addr,
            first_name,
            last_name,
            position,
            assigned_team_address: None
        }
    }
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Player {
    pub first_name: String,
    pub last_name: String,
    pub owner: Addr,
    pub position: Positions,
    pub attributes: PlayerAttributes,

}






#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub first_name: String,
    pub last_name: String,
    pub position: Positions,
    pub attributes: PlayerAttributes,
    pub managing_contract_address: Addr
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetInfo {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub player: Player,
}




