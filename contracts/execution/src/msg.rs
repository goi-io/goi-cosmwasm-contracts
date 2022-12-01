use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::messages::ManagedExecuteMsg;
use shared::task::ExecTask;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub id: String,
    pub admin: String,
    pub name: String,
    pub members: Vec<Member>,
    pub managing_contract: Option<Addr>,
    pub managing_contract_active_status: bool,
    pub for_sale: bool,
    pub price: Option<Coin>,

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ManagedServiceMessage {message: ManagedExecuteMsg},
    ExecuteTask{ exec_task: ExecTask}
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
