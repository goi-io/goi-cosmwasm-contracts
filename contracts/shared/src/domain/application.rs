use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::manage::ManagedStatus;
use crate::utils::{AppAddress, BlockTime, IManaged};
use crate::utils::general::AssetTypes;
use crate::utils::xnodes::XNodeAddress;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AppInfo {
    pub name: String,
    pub created: BlockTime,
    managed_status: ManagedStatus,
    for_sale: bool,
    asset_type: AssetTypes
}

impl AppInfo {
    // Constructs a new instance of [`Second`].
    // Note this is an associated function - no self.
    pub fn new(block_time: BlockTime) -> Self {
        Self{
            name: "".to_string(),
            created: BlockTime{
                height: block_time.height,
                time: block_time.time,
                chain_id: block_time.chain_id
            },
            managed_status: Default::default(),
            for_sale: false,
            asset_type: AssetTypes::App
        }

    }
}


impl IManaged for AppInfo {
    fn asset_type(&self) -> AssetTypes {
        self.asset_type.clone()
    }
    /*
    Struct fields:
    managed_status: ManagedStatus,
    for_sale: bool,
     */

    fn managed_status(&self) -> ManagedStatus {
        self.managed_status.clone()
    }

    fn set_managed_status(&mut self, status: ManagedStatus) -> () {
        self.managed_status = status
    }

    fn for_sale(&self) -> bool {
        self.for_sale
    }

    fn set_for_sale(&mut self, status: bool) -> () {
        self.for_sale = status;
    }
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AppTaskInfo {
    pub id: String,
    pub name: String,
    pub address: AppAddress
}





#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationQueryMsg {
    GetInfo {},
    GetTask { task_id: u8, xnode_address: Option<XNodeAddress>},


}



