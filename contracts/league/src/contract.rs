
use std::borrow::BorrowMut;
use cosmwasm_std::{Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, WasmMsg, CosmosMsg, SubMsg, ReplyOn, Coin, Timestamp};
#[cfg(not
(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw4::{Member, MemberListResponse, MemberResponse, MEMBERS_KEY, TotalWeightResponse};
use cw4_group::state::{MEMBERS, TOTAL};
use cw_storage_plus::Bound;
use goi_manager::ContractError;

use group_admin::GroupAdminError;
use group_admin::service::list_members;
use managed::ManagedServiceError;
use managed::queries::query_manageable_info;

use saleable::queries::query_saleable_info;
use shared::goi_manager::ExecuteMsg::AddPlayersToTeam;
use shared::goi_manager::{get_minters, on_successful_buy, on_successful_forsale_update, on_successful_init_processing, send_add_season_msg_to_goi_manager, send_add_team_to_league_msg_to_goi_manager, update_messaging_item_msg_to_goi_manager};
use shared::league::{LeagueInfo, set_start_and_end_date};

use shared::manage::Manageable;
use shared::player::{PlayerInfo, self};
use shared::player_attributes::{PlayerAttributes, SideOfBall};
use shared::query_response_info::{InfoManagedResponse, NameResponse};
use shared::saleable::Saleable;
use shared::season::{SeasonModelData, Season};
use shared::utils::BlockTime;
use shared::utils::general::AssetTypes;

use crate::error::LeagueError;
use crate::LeagueError::Unauthorized;
use crate::msg::{ExecuteMsg,
                 QueryMsg, InstantiateLeagueMsg};
use crate::state::{ADMIN, HOOKS, MANAGEABLE_SERVICE, SALEABLE_SERVICE, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:team";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateLeagueMsg,
) -> Result<Response, LeagueError> {

    let is_minter = get_minters().iter().any( |a| a == &Addr::unchecked(&info.sender.clone())  );

    match is_minter {
        true => {
            let state = LeagueInfo {
                address: _env.clone().contract.address,
                owner: info.sender.clone(),
                name: msg.name.clone(),
                league_type: Default::default(),
                created:  BlockTime{
                    height: _env.block.height.clone(),
                    time: _env.block.time.clone(),
                    chain_id: _env.block.chain_id.clone()
                }
            };

            set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
            STATE.save(deps.storage, &state)?;

            let init_res =
                MANAGEABLE_SERVICE.init(deps, _env.clone(), info.clone(), msg.managing_contract.clone(), AssetTypes::League, match msg.members.clone().len() > 0 {
                    true => Some(msg.members.clone()),
                    false => None
                }, Some(SALEABLE_SERVICE), Some(msg.admin.clone()), msg.price, msg.for_sale, Some(msg.name.clone()),  Some(on_successful_init_processing));
            match init_res {
                Ok(r) => {
                    Ok(r)
                }
                Err(e) => {
                    Err(LeagueError::ManagableServiceError(e))
                }
            }
        },
        false => {
            return Err(LeagueError::Unauthorized { sender: info.sender })
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, LeagueError> {

    match msg {

        ExecuteMsg::ManagedServiceMessage {  message } => {
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let res = MANAGEABLE_SERVICE.exec_msg(deps,
                                                  _env.clone(), info.clone(),
                                                  Some(SALEABLE_SERVICE), message.clone(), manager_info.clone(),
                                                  Some(on_successful_forsale_update),
                                                  Some(on_successful_buy) );

                                  
            match res {
                Ok(r) => {
                    match message {
                        managed::messages::ManagedExecuteMsg::UpdateManager { manager_address } => {
                            Ok(r)
                        }
                        _ =>  Ok(r)
                    }    
                }
                Err(e) => {
                    Err(LeagueError::ManagableServiceError(e))
                }
            }
        },
        ExecuteMsg::SetStartAndEndDate { start, end } => {
           Ok(Response::new())   //set_start_and_end_date(deps, _env,  info,start, end, &STATE)
        },
        ExecuteMsg::AddSeasonToLeague {  season_name,  season_model } => {
            ADMIN.assert_admin(deps.as_ref() , &info.clone().sender)?;
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let season = Season
                        {
                            id: 0, //managing contract will assign new id
                            league: _env.contract.address,
                            name: season_name,
                            current_episode: 0,
                            description: season_model.description.data,
                            start_date: match season_model.start_date.data { Some(t) => t, None => Timestamp::default()},
                            end_date: match season_model.end_date.data { Some(t) => t, None => Timestamp::default()},
                            access_type: season_model.access_type.data,
                            status: season_model.status.data,
                            max_teams_allowed: season_model.max_teams_allowed.data };
            let res =                 
                match  manager_info.managing_contract {
                    Some(mc) => send_add_season_msg_to_goi_manager(season, mc, Response::new()),
                    None => Response::new() ,
                };
     
            Ok(res)

        }
        ExecuteMsg::AddTeamsToLeague { team_addresses } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("League is currently unmanaged.")
                        }
                        Some(mc) => {
                          res =  send_add_team_to_league_msg_to_goi_manager(team_addresses, info.sender, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(Unauthorized { sender: info.sender})
                },
            }
        }
        ExecuteMsg::UpdateMessageStatus { message_id: invite_id, updated_message_status: updated_message_status } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("League is currently unmanaged.")
                        }
                        Some(mc) => {
                            res = update_messaging_item_msg_to_goi_manager(invite_id, updated_message_status, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(Unauthorized { sender: info.sender})
                },
            }



        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        //QueryMsg::GetTeamCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetName {} => to_binary(&query_name(deps)?),
        QueryMsg::GetInfo {} => to_binary(&query_info(deps)?),
    }
}

fn query_info(deps: Deps) -> StdResult<InfoManagedResponse<LeagueInfo>> {
    let state: LeagueInfo = STATE.load(deps.storage)?;
    let sale_info: Saleable =  (query_saleable_info(deps, SALEABLE_SERVICE)?).info;

    let managed_info: Manageable = {
        let res =
        query_manageable_info (deps, MANAGEABLE_SERVICE)?;
        res.manager
    };
    let owners = list_members(deps, None, None, MEMBERS)?;
    Ok(InfoManagedResponse { data: state, sale_info, managed_info, owners: owners.members, admin: ADMIN.get(deps)? })
}

fn query_name(deps: Deps) -> StdResult<NameResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(NameResponse { name: state.name })
}





pub fn query_total_weight(deps: Deps) -> StdResult<TotalWeightResponse> {
    let weight = TOTAL.load(deps.storage)?;
    Ok(TotalWeightResponse { weight })
}



pub fn query_member(deps: Deps, addr: String, height: Option<u64>) -> StdResult<MemberResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let weight = match height {
        Some(h) => MEMBERS.may_load_at_height(deps.storage, &addr, h),
        None => MEMBERS.may_load(deps.storage, &addr),
    }?;
    Ok(MemberResponse { weight })
}

