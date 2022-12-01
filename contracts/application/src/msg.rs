use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::messages::ManagedExecuteMsg;
use shared::task::TaskCreateModel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub app_id: String,
    pub app_name: String,
    pub admin: String,
    pub members: Vec<Member>,
    pub managing_contract: Option<Addr>,
    pub for_sale: bool,
    pub price: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ManagedServiceMessage {message: ManagedExecuteMsg},
    AddNewTask{ task: TaskCreateModel },
    UpdateTaskCodeId{ task_code_id: Option<u64>},
    AddNodeToTask { task_address: Addr }
}



// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}











