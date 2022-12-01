use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, to_binary, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;


use shared::application::ApplicationQueryMsg;
use shared::goi_manager::GoiManagerQueryMsg;
use shared::manage::ManagedStatus;
use shared::task::{TaskInfoResponse, TaskStatus};
use shared::utils::{BlockTime, ManagedItemResponse};
use shared::utils::general::AssetTypes;
use shared::utils::xnodes::XNodeStatus;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{ExecutionData, MANAGEABLE_SERVICE, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:application";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = ExecutionData {
        name: msg.name.clone(),
        create: BlockTime {
            height: _env.block.height.clone(),
            time: _env.block.time.clone(),
            chain_id: _env.block.chain_id.clone()
        },
    };
    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let init_message_res =
        MANAGEABLE_SERVICE.init(deps, _env.clone(),
                                info.clone(), msg.managing_contract.clone(), AssetTypes::Team,
                                match msg.members.clone().len() > 0 {true => Some(msg.members.clone()),false => None},
                                None, Some(msg.admin.clone()),
                                msg.price, msg.for_sale);
    match init_message_res {
        Ok(r) => {
            Ok(r)
        }
        Err(e) => {
            Err(ContractError::ManagedServiceError(e))
        }
    }


}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ManagedServiceMessage {  message } => {
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let res = MANAGEABLE_SERVICE.exec_msg(deps,
                                                  _env.clone(), info.clone(),
                                                  None,
                                                  message, manager_info);
            match res {
                Ok(r) => {
                    Ok(r)
                }
                Err(e) => {
                    Err(ContractError::ManagedServiceError(e))
                }
            }
        }
        ExecuteMsg::ExecuteTask { exec_task }=> {
            let manage_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            //let state = STATE.load(deps.storage)?;

            let get_task_msg = ApplicationQueryMsg::GetTask{
                task_id: exec_task.task_id,
                xnode_address: Some(info.sender.clone()) };
            let app_contract_res: StdResult<TaskInfoResponse> =
                deps.querier.query_wasm_smart(exec_task.application_addr.clone(), &get_task_msg);

            let get_app_managed_info_msg =  GoiManagerQueryMsg::GetManagedContract
                { contract_address: exec_task.application_addr, contract_type: AssetTypes::App };
            let get_app_managed_info_res: StdResult<Option<ManagedItemResponse>> =
                deps.querier.query_wasm_smart(manage_info.managing_contract.unwrap().clone(), &get_app_managed_info_msg);
            match get_app_managed_info_res {
                Ok(res) => {
                    match res {
                        None => {
                            return Err(ContractError::Unauthorized {})
                        }
                        Some(managed_item) => {
                            match managed_item.managed_status == ManagedStatus::Enabled {
                                true => {
                                    match app_contract_res {
                                        Ok(appInfo) => {
                                            match appInfo.task_info.status {
                                                TaskStatus::Pending => {
                                                    Err(ContractError::TaskStatusPending {})
                                                }
                                                TaskStatus::Enabled => {
                                                    match appInfo.x_node {
                                                        None => {
                                                            Err(ContractError::Unauthorized {})
                                                        }
                                                        Some(node) => {
                                                            match node.status {
                                                                XNodeStatus::Approved => {
                                                                    match node.bonded_amount[0].amount >=
                                                                        appInfo.task_info.bound_amount[0].amount &&
                                                                        node.bonded_amount[0].denom >=
                                                                            appInfo.task_info.bound_amount[0].denom {
                                                                        true => {
                                                                            //TODO: add execution code here
                                                                            let exec_message =  WasmMsg::Execute {
                                                                                contract_addr: appInfo.task_info.target_executable_contact.to_string(),
                                                                                msg: to_binary(&exec_task.exec_msg )?,
                                                                                funds: vec![]
                                                                            };

                                                                            let sub_message = SubMsg::reply_always(exec_message, 99u64);

                                                                            let response = Response::new()
                                                                                .add_submessages(vec![sub_message])
                                                                                .add_attribute("action", "ExecuteTask")
                                                                                .add_attribute("sender", info.sender.to_string());
                                                                            Ok(response)
                                                                        }
                                                                        false => {
                                                                            Err(ContractError::InsufficientBoundAmount {})
                                                                        }
                                                                    }
                                                                }
                                                                _ => {
                                                                    Err(ContractError::Unauthorized {})
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                TaskStatus::Disabled => {
                                                    Err(ContractError::TaskStatusDisabled {})
                                                }
                                                TaskStatus::Suspended => {
                                                    Err(ContractError::TaskStatusSuspended {})
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            Err(ContractError::Unauthorized {})
                                        }
                                    }
                                }
                                false => {
                                    return Err(ContractError::Unauthorized {})
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ContractError::Std(e))
                }
            }





        }
    }
}
