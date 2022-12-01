use cw_storage_plus::{Index, IndexedMap, IndexList, Item, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::service::ManagedService;
use saleable::service::SaleableService;
use shared::task::TaskData;
use shared::utils::BlockTime;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AppData {
    pub id: String,
    pub name: String,
    pub created: BlockTime,
    pub task_count: u8,
    pub task_contract_code_id: Option<u64>,
}

pub const STATE: Item<AppData> = Item::new("state");
pub const MANAGEABLE_SERVICE: ManagedService = ManagedService::new("manageable_service");
pub const SALEABLE_SERVICE: SaleableService = SaleableService::new("saleable_service");
pub const TASK_NAMESPACE: &str = "TASKS_01";


pub struct TaskIndexes<'a> {
    pub task: UniqueIndex<'a, u8, TaskData>
}

// IndexList is just boilerplate code for fetching a struct's indexes
impl<'a> IndexList<TaskData> for TaskIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<TaskData>> + '_> {
        let v: Vec<&dyn Index<TaskData>> = vec![&self.task];
        Box::new(v.into_iter())
    }
}
// tasks() is the storage access function.
pub fn tasks<'a>() -> IndexedMap<'a, &'a [u8], TaskData, TaskIndexes<'a>> {
    let indexes = TaskIndexes {
        task: UniqueIndex::new(|d| d.task_id, "task_identifier"),
    };
    IndexedMap::new(TASK_NAMESPACE, indexes)
}
