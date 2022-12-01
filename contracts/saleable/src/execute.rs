use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Addr};
use cw4::Member;
use cw_controllers::Admin;

use shared::manage::Manageable;
use shared::utils::Fee;
use shared::utils::general::AssetTypes;

use crate::error::SaleableItemError;
use crate::messages::receive::ExecuteMsg;
use crate::service::SaleableService;

pub fn execute_saleable_message(deps: DepsMut, _env: Env, info: MessageInfo,
                                saleable_service: SaleableService, msg: ExecuteMsg, admin: &Admin,
                                current_owners: Option<Vec<Member>>, fees: Option<Vec<Fee>>, manageable: Manageable)
                                -> Result<Response, SaleableItemError>
{
    match msg {
        ExecuteMsg::Buy { } => {
            let c_owners =
                match current_owners {
                    None => {
                        return Err(SaleableItemError::CurrentOwnersRequired {})
                    },
                    Some(owners) => owners
                };
            match saleable_service.execute_buy(deps.as_ref(), info.clone(),
                                               c_owners.clone(), fees) {
                Ok(res) =>
                    {
                        match SaleableService::transfer_ownership
                            (deps, _env, info, c_owners, admin) {
                            Ok(_) => {
                                Ok(res)
                            },
                            Err(e) => Err(e)
                        }
                    }
                Err(e) => Err(e)
            }
        },
        ExecuteMsg::Update { for_sale_status, price } => {
            match admin.assert_admin(deps.as_ref(), &info.sender) {
                Ok(_) =>  saleable_service.execute_update(deps, info, for_sale_status, price, manageable),
                Err(e) => Err(SaleableItemError::from(e))
            }

        },
    }

}
