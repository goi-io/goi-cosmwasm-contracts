use cosmwasm_std::{Addr, Coin, Timestamp};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::messages::ManagedExecuteMsg;
use shared::messaging::MessageTypes;
use shared::season::SeasonModelData;
use shared::utils::{MessageId, SeasonId, TeamAddr};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateLeagueMsg {
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
    SetStartAndEndDate {start: Timestamp, end: Timestamp},
    AddSeasonToLeague{  season_name: String,  season_model: SeasonModelData},
    AddTeamsToLeague {  team_addresses: Vec<TeamAddr> },
    UpdateMessageStatus {  message_id: MessageId,
        updated_message_status: MessageTypes
    },
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetName {},
    GetInfo {},
}
