use cosmwasm_std::{Addr, Binary, Coin, Deps, DepsMut, entry_point, Env, from_binary, MessageInfo, Order, Reply, Response, StdResult, SubMsg, to_binary, WasmMsg};
use cw2::set_contract_version;
use cw_storage_plus::PrimaryKey;
use cw_utils::{parse_reply_execute_data, parse_reply_instantiate_data};

use goi_manager::state::{ADMIN, MEMBERS};
use group_admin::service::list_members;
use managed::queries::query_manageable_info;
use saleable::coin_helpers::assert_sent_sufficient_coin;
use saleable::queries::query_saleable_info;
use shared::application::{ApplicationQueryMsg, AppTaskInfo};
use shared::manage::Manageable;
use shared::query_response_info::{InfoManagedResponse, InfoResponse, NameResponse};
use shared::saleable::Saleable;
use shared::task::{TaskCreateModel, TaskData, TaskInfo, TaskInfoResponse, TaskQueryMsg, TaskStatus};
use shared::utils::general::AssetTypes;
use shared::utils::{BlockTime, TaskAddress};
use shared::utils::xnodes::{XNode, XNodeAddress};
use task::state::TaskResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{AppData, MANAGEABLE_SERVICE, SALEABLE_SERVICE, STATE, tasks};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:application";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_REPLY_ID: u64 = 1u64;
const ADD_NODE_TO_TASK_REPLY_ID: u64 = 2u64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = AppData {
        id: msg.app_id.clone(),
        name: msg.app_name.clone(),
        created: BlockTime {
            height: _env.block.height.clone(),
            time: _env.block.time.clone(),
            chain_id: _env.block.chain_id.clone()
        },
        task_count: 0,
        task_contract_code_id: None
    };
    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let init_message_res =
        MANAGEABLE_SERVICE.init(deps, _env.clone(), info.clone(), msg.managing_contract.clone(), AssetTypes::App, match msg.members.clone().len() > 0
        {
            true => Some(msg.members.clone()),
            false => None
        }, Some(SALEABLE_SERVICE), Some(msg.admin.clone()), msg.price, msg.for_sale, None, None);

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
            let managable = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let res = MANAGEABLE_SERVICE.exec_msg(deps,
                                                  _env.clone(), info.clone(),
                                                  Some(SALEABLE_SERVICE),
                                                  message, managable,
                                                  None,
                                                  None);
            match res {
                Ok(r) => {
                    Ok(r)
                }
                Err(e) => {
                    Err(ContractError::ManagedServiceError(e))
                }
            }
        }
        ExecuteMsg::AddNewTask { task } => {
            add_new_task(deps, &info.sender, task )
        }
        ExecuteMsg::UpdateTaskCodeId { task_code_id } => {
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
            let mut state = STATE.load(deps.storage)?;
            state.task_contract_code_id = task_code_id;
            STATE.save(deps.storage, &state)?;
            Ok(Response::new())
        },
        ExecuteMsg::AddNodeToTask { task_address} => {
            let task =
                tasks()
                    .idx
                    .task
                    .range(deps.storage, None, None, Order::Ascending)
                    .find(|i|
                        {
                            match i.clone() {
                                Ok(record) => {
                                    match record.clone() {
                                        (_, t) => {
                                            match t.task_data {
                                                None => {
                                                    false
                                                }
                                                Some(ti) => {
                                                    ti.task_address == task_address
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    false
                                }
                            }
                        }
                    )
                    .map_or_else( || None, |i| Some(i.unwrap().1));

            match task {
                None => {
                    return Err(ContractError::TaskNotFound {})
                }
                Some(td) => {
                    let td_unwrapped = td.task_data.unwrap();
                    match td_unwrapped.status == TaskStatus::Enabled {
                        true => {
                                add_node_to_task(deps, &info.sender, info.funds, Some(td_unwrapped.bound_amount[0].clone()),
                                                 td_unwrapped.task_address, info.sender.clone())
                        }
                        false => {
                           return Err(ContractError::Unauthorized{})
                        }
                    }

                }
            }


        }
    }
}

fn add_node_to_task(deps: DepsMut, sender: &Addr, funds: Vec<Coin>, required_bond: Option<Coin> , task_address: Addr, xnode_address: Addr) -> Result<Response, ContractError>{
    assert_sent_sufficient_coin(&funds, required_bond).expect("Insufficient funds.");
    let state = STATE.load(deps.storage)?;
    match state.task_contract_code_id {
        None => {
            return Err(ContractError::TaskContractCodeIdNotSet{})
        }
        Some(_) => {

            let add_node_to_task_msg = task::msg::ExecuteMsg::AddNode {
                xnode_address
            };

            let add_node_to_task_wasm_msg = WasmMsg::Execute {
                contract_addr: task_address.to_string(),
                msg: to_binary( &add_node_to_task_msg)?,
                funds
            };
            let add_node_to_task_sub_msg = SubMsg::reply_always(add_node_to_task_wasm_msg,ADD_NODE_TO_TASK_REPLY_ID);
            Ok(Response::new()
                .add_submessage(add_node_to_task_sub_msg)
                .add_attribute("action", "add_node_to_task")
                .add_attribute("sender", sender))

        }
    }

}

fn add_new_task(deps: DepsMut, sender: &Addr,  task: TaskCreateModel) -> Result<Response, ContractError>{
    ADMIN.assert_admin(deps.as_ref(), &sender)?;
    let state = STATE.load(deps.storage)?;
    match state.task_contract_code_id {
        None => {
            return Err(ContractError::TaskContractCodeIdNotSet{})
        }
        Some(task_code_id) => {

            let new_task_msg = task::msg::InstantiateMsg{
                task_id: state.task_count.clone() as u8,
                name: task.name,
                description: task.description,
                admin: sender.to_string(),
                start_date: task.start_date,
                end_date: task.end_date,
                reward_threshold: task.reward_threshold,
                bond_amount: task.bond_amount,
                exec_msg: task.exec_msg,
                target_executable_contact: task.target_executable_contact
            };

            let instantiate_task_msg = WasmMsg::Instantiate {
                admin: None,
                code_id: task_code_id,
                msg: to_binary( &new_task_msg)?,
                funds: vec![],
                label: "task_creation".to_string()
            };


            let new_task_sub_msg = SubMsg::reply_always(instantiate_task_msg,INSTANTIATE_REPLY_ID);
            Ok(Response::new()
                .add_submessage(new_task_sub_msg)
                .add_attribute("action", "create_task")
                .add_attribute("sender", sender))
        }
    }


}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) ->  Result<Response, ContractError>{
    match reply.id {
        INSTANTIATE_REPLY_ID => handle_instantiate_reply(deps, reply),
        ADD_NODE_TO_TASK_REPLY_ID => handle_add_node_to_task_replay(deps, reply),
        id => Err(ContractError::InvalidReplyId{ message: format!("invalid reply id: {}", id) }),
    }
}

fn handle_add_node_to_task_replay(_: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let res = parse_reply_execute_data(msg);
    match res {
        Ok(r) => {
            match r.data {
                None => {
                    Err(ContractError::ReplyProcessingFailed{})
                }
                Some(_) => {
                    Ok(Response::new())

                }
            }

        }
        Err(e) => {
            Err(ContractError::ParseReplyDataError(e))
        }
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let res = parse_reply_instantiate_data(msg);
    match res {
        Ok(r) => {
            match r.data {
                None => {
                    Err(ContractError::ReplyProcessingFailed{})
                }
                Some(d) => {
                    let mut task_response_data: TaskData = from_binary(&d)?;
                    let mut state = STATE.load(deps.storage)?;

                    match task_response_data.clone().task_data {
                        None => {
                            Err(ContractError::ReplyProcessingFailed{})
                        }
                        Some(_) => {
                            let new_task_id = state.task_count + 1;
                            task_response_data.task_id = new_task_id;

                            let res = tasks().save(deps.storage,&(new_task_id).joined_key(),
                                         &task_response_data);
                            match res {
                                Ok(_) => {
                                    state.task_count = new_task_id;
                                    STATE.save(deps.storage, &state)?;
                                    Ok(Response::new())
                                }
                                Err(_) => {
                                    Err(ContractError::ReplyProcessingFailed{})
                                }
                            }

                        }
                    }

                }
            }
        }
        Err(e) => {
            Err(ContractError::ParseReplyDataError(e))

        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: ApplicationQueryMsg) -> StdResult<Binary> {
    match msg {
        //QueryMsg::GetTeamCount {} => to_binary(&query_count(deps)?),
        ApplicationQueryMsg::GetInfo {} => to_binary(&query_info(deps)?),
        ApplicationQueryMsg::GetTask { task_id, xnode_address } => {
            to_binary( &query_task(deps, _env, task_id, xnode_address)?)
        }
    }
}

fn query_info(deps: Deps) -> StdResult<InfoManagedResponse<String>> {
    //let state = STATE.load(deps.storage)?;
    let sale_info: Saleable =  (query_saleable_info(deps, SALEABLE_SERVICE)?).info;

    let managed_info: Manageable = {
        let res =
            query_manageable_info (deps, MANAGEABLE_SERVICE)?;
        res.manager
    };
    let owners = list_members(deps, None, None, MEMBERS)?;

    Ok(InfoManagedResponse { data: "".to_string(), sale_info, managed_info, owners: owners.members, admin: ADMIN.get(deps)? })
}

fn query_name(deps: Deps) -> StdResult<NameResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(NameResponse { name: state.name })
}

fn query_task(deps: Deps, env: Env, task_id: u8, xnode_addr: Option<XNodeAddress>) -> StdResult<Option<TaskInfoResponse>> {
    let state = STATE.load(deps.storage)?;

    let value = tasks().may_load(deps.storage, &([task_id]))?;
    let mut task_address: Option<TaskAddress> = Default::default();
    let task =
        match value {
            Some(t) => {
                match t.task_data {
                    None => {
                        None
                    }
                    Some(d) => {
                        task_address = Some(d.task_address.clone());
                        let get_task_info_msg = TaskQueryMsg::GetInfo { };
                        let a_task: InfoResponse<TaskResponse> = deps.querier.query_wasm_smart
                        (d.task_address, &get_task_info_msg)?;

                        Some(a_task.data.clone())
                    }
                }
            }
            None => {
                None
            }
        };

    match task {
        None => {
            Ok(None)
        }
        Some(ts) => {
            let x_node =
                match xnode_addr {
                    None => {
                        None
                    }
                    Some(x_addr) => {
                        match ts.xnodes {
                            None => {
                                None
                            }
                            Some(xnds) => {
                                match xnds.into_iter().find(|i| i.node_address == x_addr){
                                    None => {
                                        None
                                    }
                                    Some(nd) => {
                                        Some(XNode {
                                            node_address: nd.node_address,
                                            bonded_amount: nd.bonded_amount.clone(),
                                            status: nd.status.clone() })
                                    }
                                }


                            }
                        }
                    }
                };

            let res =
                    TaskInfoResponse {
                        app_info: AppTaskInfo {
                            id: state.id,
                            name: state.name,
                            address: env.contract.address
                        },
                        task_info: TaskInfo {
                            task_id,
                            task_address: match task_address {
                                None => {
                                    Addr::unchecked("")
                                }
                                Some(addr) => {
                                    addr
                                }
                            },
                            status: ts.status.clone(),
                            exec_msg: ts.exec_msg.clone(),
                            target_executable_contact: ts.target_executable_contact.clone(),
                            bound_amount: ts.bond_amount.clone()
                        },
                        x_node
                    };
            Ok( Some(res))

        }
    }
}
