use cosmwasm_std::{Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, WasmMsg, CosmosMsg, SubMsg, ReplyOn, Coin};
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
use shared::goi_manager::{get_minters, on_successful_buy, on_successful_forsale_update, on_successful_init_processing, send_request_to_cancel_season_spot, send_request_to_join_open_season, send_request_to_join_winner_takes_all_season, update_messaging_item_msg_to_goi_manager};

use shared::manage::Manageable;
use shared::player::{PlayerInfo, self};
use shared::player_attributes::{PlayerAttributes, SideOfBall};
use shared::query_response_info::{InfoManagedResponse, NameResponse};
use shared::saleable::Saleable;
use shared::utils::general::AssetTypes;

use crate::error::TeamError;
use crate::msg::{ExecuteMsg, InstantiateTeamMsg, PlayerResponse, PlayersResponse,
                 QueryMsg};
use crate::state::{ADMIN, HOOKS, MANAGEABLE_SERVICE, SALEABLE_SERVICE, State, STATE};
use crate::team_attributes::TeamPlayers;
use crate::TeamError::UnauthorizedSender;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:team";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateTeamMsg,
) -> Result<Response, TeamError> {

    let is_minter = get_minters().iter().any( |a| a == &Addr::unchecked(&info.sender.clone())  );

    match is_minter {
        true => {
            let state = State {
                name: msg.name.clone(),
                players: TeamPlayers {
                    rb: None,
                    qb: None,
                    wr1: None,
                    wr2: None,
                    co: None,
                    gl: None,
                    gr: None,
                    s: None,
                    cb1: None,
                    cb2: None,
                    lb: None,
                    cd: None,
                    tr: None,
                    tl: None
                },
            };

            set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
            STATE.save(deps.storage, &state)?;

            let init_res =
                MANAGEABLE_SERVICE.init(deps, _env.clone(), info.clone(), msg.managing_contract.clone(), AssetTypes::Team, match msg.members.clone().len() > 0 {
                    true => Some(msg.members.clone()),
                    false => None
                }, Some(SALEABLE_SERVICE), Some(msg.admin.clone()), msg.price, msg.for_sale, Some(msg.name.clone()),  Some(on_successful_init_processing));
            match init_res {
                Ok(r) => {
                    Ok(r)
                }
                Err(e) => {
                    Err(TeamError::ManagableServiceError(e))
                }
            }
        },
        false => {
            return Err(TeamError::Unauthorized { })
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, TeamError> {

    match msg {
        ExecuteMsg::ManagedServiceMessage { message } => {
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let res = MANAGEABLE_SERVICE.exec_msg(deps,
                                                  _env.clone(), info.clone(),
                                                  Some(SALEABLE_SERVICE), message.clone(), manager_info.clone(),
                                                  Some(on_successful_forsale_update),
                                                  Some(on_successful_buy));


            match res {
                Ok(r) => {
                    match message {
                        managed::messages::ManagedExecuteMsg::UpdateManager { manager_address } => {
                            Ok(r)//todo!()
                        }
                        _ => Ok(r)// todo!()
                    }
                }
                Err(e) => {
                    Err(TeamError::ManagableServiceError(e))
                }
            }
        },
        ExecuteMsg::AddPlayersToTeam { players: pls } => {
            add_players_to_team(deps, &info.sender, pls)
        },
        ExecuteMsg::RemovePlayersFromTeam { players: pls } => {
            remove_players_from_team(deps, &info.sender, pls)
        },
        ExecuteMsg::UpdateMessageStatus { message_id: invite_id, updated_message_status: updated_invite_message_status } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("Team is currently unmanaged.")
                        }
                        Some(mc) => {
                            res = update_messaging_item_msg_to_goi_manager( invite_id, updated_invite_message_status, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(UnauthorizedSender { sender: info.sender })
                },
            }
        }
        ExecuteMsg::JoinLeague { season_id } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("Team is currently unmanaged.")
                        }
                        Some(mc) => {
                            res = send_request_to_join_open_season(season_id, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(UnauthorizedSender { sender: info.sender })
                },
            }

        },
        ExecuteMsg::JoinLeagueWinnerTakeAll { season_id, fee } => {

            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("Team is currently unmanaged.")
                        }
                        Some(mc) => {
                            res = send_request_to_join_winner_takes_all_season(season_id, fee, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(UnauthorizedSender { sender: info.sender })
                },
            }
        },
        ExecuteMsg::Deposit {  } => {
            Err(UnauthorizedSender { sender: info.sender })
        }
        ExecuteMsg::CancelSeasonSpot { season_id } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let mut res = Response::new();
            match is_admin_res {
                Ok(_) => {
                    match manager_info.managing_contract {
                        None => {
                            panic!("Team is currently unmanaged.")
                        }
                        Some(mc) => {
                            res = send_request_to_cancel_season_spot(season_id, mc, res.clone());
                        }
                    }
                    Ok(res)
                },
                Err(_) => {
                    Err(UnauthorizedSender { sender: info.sender })
                },
            }
        }
    }
}







pub fn add_players_to_team( deps: DepsMut, sender: &Addr, players: Vec<PlayerInfo>) -> Result<Response, TeamError> {
    ADMIN.assert_admin(deps.as_ref(), &sender)?;
    let mut add_players_to_managing_contract = false;
    let mut managing_contract_address = "".to_string();
    //TODO: add managing contract status check
    match verify_position_assignments(&deps, players.clone()) {
        Ok(items) => {
            let manager_info = MANAGEABLE_SERVICE.get(deps.as_ref())?;
            let add_player_to_manager_res =
                match manager_info.managing_contract {
                    None  => {
                        Err(TeamError::Unauthorized {})
                    },
                    Some(mc) => {
                        add_players_to_managing_contract = true;
                        managing_contract_address = mc.into_string();
                        Ok(())
                    }
                };
            match add_player_to_manager_res {
                Ok(_) => {
                    let res =
                        STATE.update(deps.storage, |mut state| -> Result<_, TeamError> {
                            let mut update_res: Result<_, TeamError> = Ok(());
                            for i in items {
                                update_res = state.players.add_player_to_team(i.address.clone(), i.position.clone());
                                match update_res {
                                    Ok(_) => continue,
                                    Err(_) => break
                                }
                            }
                            match update_res {
                                Ok(_) => Ok(state),
                                Err(e) => Err(e)
                            }
                        });
                    match res {
                        Ok(_) => {
                                match add_players_to_managing_contract {
                                    true => {
                                         let exec_message =  WasmMsg::Execute {
                                                contract_addr: managing_contract_address,
                                                msg: to_binary(&AddPlayersToTeam { players })?,
                                                funds: vec![]
                                         };

                                         Ok(Response::new()
                                             .add_message(exec_message)
                                             .add_attribute("action", "create_players")
                                             .add_attribute("sender", sender))
                                    }
                                    false => {
                                        Err(TeamError::Unauthorized {})
                                    }
                                }

                            },
                        Err(e) => Err(e)

                    }
                }
                Err(e) => {
                    Err(e)
                }
            }

        },
        Err(ea) => {
            match ea {
                TeamError::Std(e) => {
                    match e {

                        StdError::GenericErr { .. } => {

                                Err(TeamError::GenericErr { message: "".to_string() })

                        }
                        _ => Err(TeamError::Std(e))

                    }

                },

                _ => Err(ea)
            }


        },
    }
}




fn verify_position_assignments(deps: &DepsMut, players: Vec<PlayerInfo>) -> Result<Vec<PlayerInfo> , TeamError>{
    let mut res: Result<(), TeamError>  = Ok(());
    let pl_msg = player::QueryMsg::GetInfo {};
    let mut hold: Vec<PlayerInfo> = Vec::default();
    {
        let mut state = STATE.load(deps.storage)?;
        for players_to_add in players {
            let player_from_contract: player::InfoResponse =  deps.querier.query_wasm_smart(players_to_add.address.clone(), &pl_msg)?;

            //check whether requested player's position actually matches
            // what player's position is on contract
            match player_from_contract.player.position == players_to_add.position {
                true =>
                    match state.players.get_player_at_position(players_to_add.position.clone()) {
                        Some(_) => res = Err(TeamError::PositionAlreadyAssigned{}),
                        None => res ={
                            hold.push(players_to_add);
                            Ok(())
                        }
                    },
                false => res = Err(TeamError::PlayerDeclaredPosAndAssignPosMisMatch{
                    player_address: players_to_add.address,
                    position: players_to_add.position,
                    contract_first_name: player_from_contract.player.first_name,
                    contract_last_name: player_from_contract.player.last_name,
                    contract_position: player_from_contract.player.position
                })
            };
            match res {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }
    match res {
        Ok(_) =>{
                match hold.len() > 0 {
                    true => Ok(hold),
                    false => Err(TeamError::PositionAssignmentsNotProvided {})
                }
        },
        Err(e) => Err(e)
    }
}


pub fn remove_players_from_team(deps: DepsMut, sender: &Addr, players:Vec<PlayerInfo>) -> Result<Response, TeamError> {
    ADMIN.assert_admin(deps.as_ref(), &sender)?;
    let res =
            STATE.update(deps.storage, |mut state| -> Result<_, TeamError> {
                match state.players.remove_players_from_positions(players) {
                    Ok(_) => Ok(state),
                    Err(e) => Err(e)
                }
            });
    match res {
        Ok(_) => Ok(Response::new().add_attribute("method", "remove_players_from_team")),
        Err(e) => Err(e)
    }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        //QueryMsg::GetTeamCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetName {} => to_binary(&query_name(deps)?),
        QueryMsg::GetPlayer { addr } =>  {
            let p_addr = deps.api.addr_validate(&*addr)?;
            to_binary( &query_player(deps, p_addr)?)
        },
        QueryMsg::GetDefense {} => to_binary(&query_defense_players(deps)?),
        QueryMsg::GetOffense {} => to_binary(&query_offense_players(deps)?),
        QueryMsg::GetAllPlayers {} => to_binary(&query_all_players(deps)?),
        QueryMsg::GetInfo {} => to_binary(&query_info(deps)?),
    }
}

fn query_info(deps: Deps) -> StdResult<InfoManagedResponse<State>> {
    let state: State = STATE.load(deps.storage)?;
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


fn query_all_players(deps: Deps) -> StdResult<PlayersResponse> {
    let mut state = STATE.load(deps.storage)?;
    Ok(PlayersResponse { players: state.players.all_players() })
}


fn query_player(deps: Deps, player_address: Addr) -> StdResult<PlayerResponse> {
    let mut state = STATE.load(deps.storage)?;
    Ok(PlayerResponse { players: state.players.get_player(player_address) })
}


fn query_defense_players(deps: Deps) -> StdResult<PlayersResponse> {
    let mut state = STATE.load(deps.storage)?;
    Ok(PlayersResponse
    { players: state.players.get_players_by_side_of_players(SideOfBall::Defense) })
}

fn query_offense_players(deps: Deps) -> StdResult<PlayersResponse> {
    let mut state = STATE.load(deps.storage)?;
    Ok(PlayersResponse
    { players: state.players.get_players_by_side_of_players(SideOfBall::Offense) })
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

