use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shared::utils::{BlockTime, JsonData};
use shared::utils::xnodes::SuccessfulExecutionCount;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub task_id: u8,
    pub name: String,
    pub description: Option<String>,
    pub admin: String,
    pub start_date: BlockTime,
    pub end_date: Option<BlockTime>,
    pub reward_threshold: SuccessfulExecutionCount,
    pub bond_amount: Vec<Coin>,
    pub exec_msg: Option<JsonData>,
    pub target_executable_contact: Addr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddNode{ xnode_address: Addr}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
