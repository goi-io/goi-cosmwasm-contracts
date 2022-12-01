use cosmwasm_std::{Addr, Deps, StdResult};
use shared::manage::{ManagedContract, ManagementFee};
use shared::manage::receive::{ManagedContractInfoResponse, ManagementInfoResponse};


use crate::service::ManagementService;

pub fn query_management_info(deps: Deps, management_item: ManagementService)
    -> StdResult<ManagementInfoResponse> {
    let res: Vec<ManagementFee> =
        management_item.get(deps).unwrap().fees.into_iter()
        .filter(|i|i.active)
        .collect();
    let fees =
        match res.len()> 0 {
            true =>  Some(res),
            false => None
        };
    Ok(ManagementInfoResponse{ fees })
}





