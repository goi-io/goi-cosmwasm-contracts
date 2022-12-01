use cosmwasm_std::{Deps, StdResult};

use crate::messages::receive::GetInfoResponse;
use crate::service::SaleableService;

pub fn query_saleable_info(deps: Deps, saleable_service: SaleableService) -> StdResult<GetInfoResponse> {
    Ok(GetInfoResponse{ info: saleable_service.get(deps)?})
}
