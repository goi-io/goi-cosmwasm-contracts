use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::service::ManagedService;
use shared::utils::BlockTime;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecutionData {
    pub name: String,
    pub create: BlockTime,

}

pub const STATE: Item<ExecutionData> = Item::new("state");
pub const MANAGEABLE_SERVICE: ManagedService = ManagedService::new("manageable_service");

