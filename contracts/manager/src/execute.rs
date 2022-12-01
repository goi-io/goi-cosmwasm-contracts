use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_controllers::Admin;
use shared::manage::receive::ExecuteMsg;

use crate::error::ManagementError;

use crate::service::ManagementService;

pub fn execute_management_message(deps: DepsMut, _env: Env, info: MessageInfo,
                                  management_service: ManagementService, msg: ExecuteMsg, _ : &Admin)
                                  -> Result<Response, ManagementError>
{
    match msg {
        ExecuteMsg::UpdateFees { add, remove }
        => management_service.update_fees(deps, info, _env, add, remove)
    }

}
