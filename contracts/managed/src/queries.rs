use cosmwasm_std::{Deps, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shared::manage::Manageable;

use crate::service::ManagedService;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ManageableInfoResponse {
    pub manager: Manageable,
}

pub fn query_manageable_info(deps: Deps, manageable_item: ManagedService)
    -> StdResult<ManageableInfoResponse>
{

    Ok(ManageableInfoResponse{
        manager: manageable_item.get(deps)?
    })

}
