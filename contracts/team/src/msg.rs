use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::messages::ManagedExecuteMsg;
use shared::messaging::MessageTypes;
use shared::player::PlayerInfo;
use shared::utils::{MessageId, SeasonId};

use crate::team_attributes::TeamPosition;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateTeamMsg {
    pub name: String,
    pub admin: String,
    pub members: Vec<Member>,
    pub managing_contract: Option<Addr>,
    //pub managing_contract_active_status: bool,
    pub for_sale: bool,
    pub price: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ManagedServiceMessage {message: ManagedExecuteMsg},
    AddPlayersToTeam { players: Vec<PlayerInfo> },
    RemovePlayersFromTeam {players: Vec<PlayerInfo> },
    //Manage { manageable_msg: ManageableExecuteMsg},
    UpdateMessageStatus {  message_id: MessageId,
        updated_message_status: MessageTypes
    },

    JoinLeague { season_id: SeasonId },
    CancelSeasonSpot { season_id: SeasonId },
    JoinLeagueWinnerTakeAll { season_id: SeasonId, fee: Vec<Coin> },
    Deposit{}

}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetName {},
    GetPlayer{addr: String},
    GetAllPlayers {},
    GetOffense {},
    GetDefense {},
    GetInfo {},
}






#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayersResponse {
    pub players:  Vec<TeamPosition>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerResponse {
    pub players: Option<TeamPosition>,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub team: State,
    pub sale_info: Saleable,
    pub managed_info: Manageable,
    pub owners: Vec<Member>,
    pub admin: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NameResponse {
    pub name: String,
}

 */
