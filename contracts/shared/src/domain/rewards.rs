use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::utils::BlockTime;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
    pub app_id: String,
    pub app_address: Addr,
    pub user_owner: Addr,
    pub claimed: Option<BlockTime>,
    pub earned: BlockTime,
    pub reward_type: RewardTypes,
}



#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, JsonSchema)]
#[repr(u8)]
pub enum RewardTypes {
    NA = 0,
    Other = 1,
    Team = 2,
    League = 3,

}

impl Default for RewardTypes {
    fn default() -> Self {
        RewardTypes::NA
    }
}


