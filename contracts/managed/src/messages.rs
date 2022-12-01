use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use group_admin::messages::receive::ExecuteMsg as GroupAdminHooksMsg;
use saleable::messages::receive::ExecuteMsg as SaleableExecuteMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManagedExecuteMsg {
    Saleable{ saleable_msg: SaleableExecuteMsg},
    UpdateManager{ manager_address: Addr},
    GroupAdminHooks {group_admin_hooks_msg: GroupAdminHooksMsg  },

}
