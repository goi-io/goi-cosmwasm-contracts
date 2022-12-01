use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::application::AppTaskInfo;
use crate::utils::{AppAddress, BlockTime, ContractAddress, JsonData, TaskAddress};
use crate::utils::xnodes::{SuccessfulExecutionCount, XNode};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecTask {
    pub task_id: u8,
    pub task_address: TaskAddress,
    pub application_addr: AppAddress,
    pub exec_msg: JsonData,

}




#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, JsonSchema)]
#[repr(u8)]
pub enum TaskStatus {
    Pending = 0,
    Enabled = 1,
    Disabled = 2,
    Suspended = 3,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskInfo {
    pub task_id: u8,
    pub task_address: TaskAddress,
    pub status: TaskStatus,
    pub exec_msg: Option<JsonData>,
    pub target_executable_contact: Addr,
    pub bound_amount: Vec<Coin>


}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskCreateModel {
    pub name: String,
    pub description: Option<String>,
    pub start_date: BlockTime,
    pub end_date: Option<BlockTime>,
    pub reward_threshold: SuccessfulExecutionCount,
    pub bond_amount: Vec<Coin>,
    pub exec_msg: Option<JsonData>,
    pub target_executable_contact: ContractAddress,

    pub task_id: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskInfoResponse {
    pub app_info: AppTaskInfo,
    pub task_info: TaskInfo,
    pub x_node: Option<XNode>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskData {
    pub task_id: u8,
    pub name: String,
    pub description: Option<String>,
    pub task_data: Option<TaskInfo>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskQueryMsg {
    GetInfo {},
}
