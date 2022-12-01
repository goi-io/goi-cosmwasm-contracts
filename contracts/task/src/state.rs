use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Index, IndexedMap, IndexList, Item, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shared::rewards::RewardTypes;
use shared::task::TaskStatus;
use shared::utils::{AppAddress, BlockTime, ContractAddress, JsonData};
use shared::utils::xnodes::{SuccessfulExecutionCount, XNode};

pub const STATE: Item<Task> = Item::new("state");
pub const NODE_NAMESPACE: &str = "NODES_01";

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub bond_amount: Vec<Coin>,
    pub application_addr: AppAddress,
    pub start_date: BlockTime,
    pub end_date: Option<BlockTime>,
    pub exec_msg: Option<JsonData>,
    pub target_executable_contact: ContractAddress,
    pub reward_type: RewardTypes,
    pub reward_threshold: SuccessfulExecutionCount,
    pub status: TaskStatus,
}


#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskResponse {
    pub name: String,
    pub description: Option<String>,
    pub bond_amount: Vec<Coin>,
    pub application_addr: AppAddress,
    pub start_date: BlockTime,
    pub end_date: Option<BlockTime>,
    pub exec_msg: Option<JsonData>,
    pub target_executable_contact: ContractAddress,
    pub reward_type: RewardTypes,
    pub reward_threshold: SuccessfulExecutionCount,
    pub status: TaskStatus,
    //holds applied/approved execution nodes
    pub xnodes: Option<Vec<XNode>>
}

pub struct XnodeIndexes<'a> {
    pub xnode: UniqueIndex<'a, Addr, XNode>
}


// IndexList is just boilerplate code for fetching a struct's indexes
impl<'a> IndexList<XNode> for XnodeIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<XNode>> + '_> {
        let v: Vec<&dyn Index<XNode>> = vec![&self.xnode];
        Box::new(v.into_iter())
    }
}

// tasks() is the storage access function.
pub fn xnodes<'a>() -> IndexedMap<'a, &'a [u8], XNode, XnodeIndexes<'a>> {
    let indexes = XnodeIndexes {
        xnode: UniqueIndex::new(|d|d.node_address.clone(), "node_identifier"),
    };
    IndexedMap::new(NODE_NAMESPACE, indexes)
}
