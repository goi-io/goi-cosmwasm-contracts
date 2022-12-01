use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw4_group::contract::execute_update_members;
use cw_controllers::{Admin, Hooks};

use crate::GroupAdminError;
use crate::messages::receive::ExecuteMsg;

pub fn execute_group_admin_message(deps: DepsMut, _env: Env, info: MessageInfo,
                                   msg: ExecuteMsg, a_admin: &Admin, hooks: Hooks)
                                   -> Result<Response, GroupAdminError>
{
    let api = deps.api;
    match msg {
        ExecuteMsg::RemoveHook { addr } => {
            match hooks.execute_remove_hook
            (&a_admin, deps, info, api.addr_validate(&addr)?) {
                Ok(r) => Ok(r),
                Err(e) =>  Err(GroupAdminError::HookError(e))
            }
        }
        ExecuteMsg::AddHook { addr } => {
            match hooks.execute_add_hook
            (&a_admin, deps, info, api.addr_validate(&addr)?) {
                Ok(r) => Ok(r),
                Err(e) => Err(GroupAdminError::HookError(e))
            }

        },
        ExecuteMsg::UpdateAdmin { admin_addr } => {
            let res =
                a_admin.execute_update_admin(
                    deps,
                    info,
                    admin_addr
                        .map(|admin| api.addr_validate(&admin)).transpose()?,
                );
            match res {
                Ok(r) => Ok(r),
                Err(e) =>  Err(GroupAdminError::AdminError(e))
            }
        }
        ExecuteMsg::UpdateMembers { remove, add } => {
            match execute_update_members(deps, _env, info, add, remove) {
                Ok(t) => Ok(t),
                Err(e) => Err(GroupAdminError::GroupError(e))
            }
        }
    }

}


