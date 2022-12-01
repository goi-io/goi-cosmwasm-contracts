use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use shared::query_response_info::{InfoResponse, NameResponse};
use shared::task::{TaskData, TaskInfo, TaskQueryMsg, TaskStatus};
use shared::utils::xnodes::XNode;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{STATE, Task, TaskResponse, xnodes};

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
    let state = Task {
        name: msg.name.clone(),
        description: msg.description.clone(),
        bond_amount: msg.bond_amount.clone(),
        application_addr: info.sender.clone(),
        start_date: msg.start_date.clone(),
        end_date: msg.end_date.clone(),
        exec_msg: msg.exec_msg.clone(),
        target_executable_contact: msg.target_executable_contact.clone(),
        reward_type: Default::default(),
        reward_threshold: msg.reward_threshold.clone(),
        status: TaskStatus::default(),
    };

    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let res_data = TaskData {
        task_id: msg.task_id.clone(),
        name: msg.name.clone(),
        description: msg.description.clone(),
        task_data: Some(TaskInfo {
            task_id: msg.task_id.clone(),
            task_address: _env.contract.address.clone(),
            status: Default::default(),
            exec_msg: msg.exec_msg.clone(),
            target_executable_contact: msg.target_executable_contact.clone(),
            bound_amount: msg.bond_amount.clone()
        })
    };

    Ok(Response::new().set_data( to_binary(&res_data)?))

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddNode { xnode_address } => {
            let state = STATE.load(deps.storage)?;
            match info.sender == xnode_address {
                true => {

                }
                false => {}
            }
        }
    }
    Ok(Response::new())
}




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: TaskQueryMsg) -> StdResult<Binary> {
    match msg {
        TaskQueryMsg::GetInfo { } => {
           to_binary( &query_info(deps)?)
        }
    }

}

fn query_name(deps: Deps) -> StdResult<NameResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(NameResponse { name: state.name })
}

fn query_info(deps: Deps) -> StdResult<InfoResponse<TaskResponse>> {
    let state = STATE.load(deps.storage)?;


    let nodes = {
        let nds: Vec<XNode> =
            xnodes()
                .idx
                .xnode
                .range(deps.storage, None, None, Order::Ascending)
                .map(|i| i.unwrap().1)
                .collect();
        match nds.len() > 0 {
            true => {
                Some(nds)
            }
            false => {
                None
            }
        }
    };


    let res = TaskResponse {
        name: state.name.to_string(),
        description: state.description,
        bond_amount: state.bond_amount,
        application_addr: state.application_addr,
        start_date: state.start_date,
        end_date: state.end_date,
        exec_msg: state.exec_msg,
        target_executable_contact: state.target_executable_contact,
        reward_type: state.reward_type,
        reward_threshold: state.reward_threshold,
        status: state.status,
        xnodes: nodes

    };
    Ok(InfoResponse{ data: res })
}

