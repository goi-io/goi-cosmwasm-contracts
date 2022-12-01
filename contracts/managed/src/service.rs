use std::fmt;

use cosmwasm_std::{CosmosMsg, Env, QuerierWrapper, ReplyOn, SubMsg, to_binary};
use cosmwasm_std::{Addr, attr, Coin, Deps, DepsMut, MessageInfo,
                   Response, StdResult, Storage};
use cw4::Member;
use cw_storage_plus::Item;
use schemars::JsonSchema;



use goi_manager::state::{ADMIN, HOOKS, MEMBERS};
use group_admin::execute::execute_group_admin_message;
use group_admin::service::{initialize_members_and_admin, list_members, validate_owner_count};
use shared::manage::{ManagedStatusChangedHookMsg, ManagedStatusUpdate, ManagementFee, OnSuccessfulBuyExec, OnSuccessfulInit, OnSuccessfulForSaleStatusUpdateExec};
use shared::manage::receive::ManagementInfoResponse;

use saleable::execute::execute_saleable_message;
use saleable::messages::receive::ExecuteMsg;
use saleable::service::SaleableService;
use shared::goi_manager::GoiManagerQueryMsg;
use shared::goi_manager::ManagementQryMsg::GetManagementInfo;
use shared::manage::{Manageable, ManagedStatus};
use shared::saleable::Saleable;
use shared::utils::general::AssetTypes;

use crate::error::ManagedServiceError;
use crate::messages::ManagedExecuteMsg;



// state/logic
pub struct ManagedService<'a>(Item<'a, Manageable>);

impl<'a> ManagedService<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        ManagedService(Item::new(storage_key))
    }

    pub fn save(&self, storage: &mut dyn Storage, manageable: Manageable) -> StdResult<()> {
        self.0.save(storage, &manageable)
    }

    pub fn get(&self, deps: Deps) -> StdResult<Manageable> {
        self.0.load(deps.storage)
    }

    pub fn init(&self, deps: DepsMut, env: Env, info: MessageInfo,
                managing_contract: Option<Addr>,
                managed_asset_type: AssetTypes,
                members: Option<Vec<Member>>,
                saleable_service: Option<SaleableService>, admin: Option<String>,
                price: Option<Coin>,
                for_sale: bool, asset_name: Option<String>, on_succesful_init: Option<OnSuccessfulInit>) ->Result<Response, ManagedServiceError> {
        match validate_owner_count(deps.as_ref(), None,members.clone()) {
            Ok(_) => (),
            Err(e) => return Err( ManagedServiceError::GroupAdminHooksError(e))
        }

        match saleable_service {
            None => {
                ()
            }
            Some(ss) => {
                ss.save(deps.storage, Saleable {
                    price_version: 0,
                    price,
                    for_sale
                }).expect("Save saleable state failed!!");
            }
        }


        self.save(deps.storage,
                  Manageable { managing_contract: managing_contract.clone(),  managed_asset_type: managed_asset_type.clone()}).expect("Save manageable state failed!!");


        match managing_contract.clone() {
            None => (),
            Some(manager) => {
                let _ = HOOKS.add_hook(deps.storage, manager);


            }
        }
        let res =
            match members.clone() {
                None => {
                    Ok(Response::new())
                }
                Some(mems) => {
                    match initialize_members_and_admin(deps, env, info.clone(),admin.clone(), mems) {
                        Ok(r) => Ok(r),
                        Err(e) => {
                            Err(ManagedServiceError::GroupAdminHooksError(e))
                        }
                    }
                }
            };
        match res {
            Ok(r) => {
                match on_succesful_init {
                    None => {
                        Ok(r)
                    }
                    Some(aFn) => {
                        match managing_contract.clone() {
                            None => {
                                Ok(r)
                            }
                            Some(mc) => {
                                Ok(aFn(asset_name, info.sender.clone(), managed_asset_type, mc, r))
                            }
                        }

                    }
                }
            }
            Err(e) => {Err(e)}
        }
    }

    pub fn exec_msg (&self, deps: DepsMut, env: Env, info: MessageInfo, s_service: Option<SaleableService>,
                     exec_msg: ManagedExecuteMsg, manageable: Manageable,
                     on_succesfull_for_sale_status_update_exec: Option<OnSuccessfulForSaleStatusUpdateExec>,
                     on_succesful_buy_exec: Option<OnSuccessfulBuyExec>) -> Result<Response, ManagedServiceError> {
            match exec_msg {
                ManagedExecuteMsg::Saleable { saleable_msg } => {
                    match s_service {
                        None => {
                            return Err(ManagedServiceError::SaleServiceNotEnabled{})
                        }
                        Some(ss) => {
                            let owners = list_members(deps.as_ref(), None, None, MEMBERS)?;
                            let mang = self.get(deps.as_ref())?;
                            let fees = {
                                let res_fees = self.get_management_fees(mang, &deps.querier);
                                match res_fees {
                                    None => None,
                                    Some(rf) => {
                                        let res =
                                            rf.into_iter()
                                                .map(|i| i.fees)
                                                .collect();
                                        Some(res)
                                    }
                                }
                            };
                            match execute_saleable_message
                                ( deps, env, info.clone(), ss,
                                  saleable_msg.clone(), &ADMIN, Some(owners.members), fees, manageable.clone())
                            {

                                Ok(r) =>
                                    {
                                        let managing_contract_addr = manageable.managing_contract.clone();
                                        match saleable_msg.clone() {
                                            ExecuteMsg::Buy {} => {
                                                match (on_succesful_buy_exec, managing_contract_addr.clone()) {
                                                    (Some(aFn), Some(mc)) => {
                                                       Ok( aFn(info.sender.clone(), mc, r))
                                                    },
                                                    _ =>  Ok(r),
                                                }
                                               
                                            }
                                            ExecuteMsg::Update { for_sale_status, price } => {
                                                match (on_succesfull_for_sale_status_update_exec, managing_contract_addr.clone()) {
                                                    (Some(aFn), Some(mc) )=> {
                                                       Ok( aFn(for_sale_status, price, mc, r))
                                                    },
                                                    _ =>  Ok(r),
                                                }
                                            },
                                     
                                        }

                                    },
                                Err(e) => Err(ManagedServiceError::SaleItemErrors(e))
                            }
                        }
                    }

                },
                ManagedExecuteMsg::UpdateManager { manager_address } => {
                    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
                    match self.update_manager
                    (deps, info, manager_address) {
                        Ok(r) => Ok(r),
                        Err(e) => Err(e)
                    }
                },
                ManagedExecuteMsg::GroupAdminHooks { group_admin_hooks_msg } => {
                    match execute_group_admin_message(
                        deps, env, info, group_admin_hooks_msg, &ADMIN, HOOKS) {
                        Ok(r) => Ok(r),
                        Err(e) => Err(ManagedServiceError::GroupAdminHooksError(e))
                    }
                }
            }

    }


    pub fn update_manager<C>(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        update_manager_address: Addr,
    ) -> Result<Response<C>, ManagedServiceError>
        where
            C: Clone + fmt::Debug + PartialEq + JsonSchema,
    {
        //let's get the original asset_type information and retain it
        //as assets' type never change
        let current_state = self.get(deps.as_ref())?;
        self.save(deps.storage,
                  Manageable {
                      managing_contract: Some(update_manager_address.clone()),
                      managed_asset_type: current_state.managed_asset_type,
                  }
        ).expect("Failed to update manager.");

        Ok(Response::new().
            add_attributes(vec![
                attr("action", "update_manager"),
                attr("sender", info.sender),
                attr("manager", update_manager_address.clone()),
            ]))
    }

    pub fn get_management_fees(&self, mangeable: Manageable, q_wrapper: &QuerierWrapper)
                           -> Option<Vec<ManagementFee>>  {
        match mangeable.managing_contract {
            None => None,
            Some(managing_contract_address) => {
                let res: ManagementInfoResponse = q_wrapper.query_wasm_smart
                (managing_contract_address.clone(),
                 &GoiManagerQueryMsg::ManagementQryMessages
                 { management_qry_msg: GetManagementInfo {} }).unwrap();
                res.fees
            }
        }
    }

    pub fn update_manager_status(&self,
                                    deps: DepsMut,
                                    _env: Env,
                                    active_status: ManagedStatus,
    ) -> Result<ManagedStatusChangedHookMsg, ManagedServiceError>
    {
        let current_state = self.get(deps.as_ref())?;

        match current_state.clone().managing_contract {
            None => return Err(ManagedServiceError::NoManagerContractAddressProvided {}),
            Some(manager_addr) => {

                self.save(deps.storage, current_state).expect("Failed to update manager status.");
                Ok(ManagedStatusChangedHookMsg {
                    change: ManagedStatusUpdate {
                        managed_contract: _env.contract.address.to_string(),
                        manager_contract: manager_addr.to_string(),
                        managed_status: active_status
                    }
                })
            }
        }
    }
}



