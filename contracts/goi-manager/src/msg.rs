use std::collections::HashMap;

use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use group_admin::messages::receive::ExecuteMsg as GroupAdminHooksMsg;
use shared::manage::ManagementFee;

use shared::player::PlayerInfo;
use shared::utils::general::AssetTypes;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub admin: String,
    pub members: Vec<Member>,
    pub teams: Option<HashMap<Addr, Vec<Addr>>>,
    pub teams_for_sale: Option<Vec<Addr>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    WithdrawFundsToCommunityPool { address: String },
}







