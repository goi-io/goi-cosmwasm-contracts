use cosmwasm_std::{Addr, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::utils::{EndDate, MessageId, SeasonId};
use crate::utils::general::AssetTypes;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct JoinSeasonRequestInfo {
    pub status_type: MessageTypes,
    pub season_id: SeasonId,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageTypes {
    CancelSeason {},
    Accepted {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DeliveryPacket {
        pub asset_type: AssetTypes,
        pub address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DeliveryInfo {
    pub to: DeliveryPacket,
    pub from: DeliveryPacket
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Message<T> {
    pub id: MessageId,
    pub updated: Timestamp,
    pub created: Timestamp,
    pub delivery: DeliveryInfo,
    pub data:  T,
    pub notes: Vec<u8>
}

