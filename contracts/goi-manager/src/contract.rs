
use std::borrow::{Borrow, BorrowMut};
use std::clone;
use std::collections::HashSet;
use std::ops::Add;
use std::str::FromStr;

use cosmwasm_std::{Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Coin, BlockInfo, StdError, Storage, Order, CosmosMsg, SubMsg, ReplyOn, Uint128, BankMsg, Timestamp};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::testing::mock_env;
use cw2::set_contract_version;


use cw_storage_plus::{Item, Map, MultiIndex, PrefixBound};
use StdError::NotFound;
use group_admin::execute::execute_group_admin_message;
use group_admin::service::initialize_members_and_admin;
use crate::msg::{SudoMsg as GoiSudoMsg};


use manager::service::ManagementService;
use shared::manage::receive::{ManagedContractInfoResponse};
use manager::queries::{  query_management_info};
use shared::goi_manager::{GoiMangerContractModel, GoiManagerQueryMsg, ExecuteMsg, get_minters, ManagementQryMsg};
use shared::GoiError;
use shared::league::LeagueTeamAssignment;
use shared::manage::{Management, ManagementFee, ManagedStatus, ManagedContract, ManagedStatusChangedHookMsg};
use shared::messaging::{DeliveryInfo, DeliveryPacket, JoinSeasonRequestInfo, Message, MessageTypes};
use shared::player::{PlayerInfo, PlayerInfoPacket};


use shared::rewards::{Reward, RewardTypes};
use shared::season::{Season, SeasonAccessTypes, SeasonLedger, SeasonModelData, SeasonStatus};
pub use shared::team::{TeamInfo};
use shared::utils::{Fee, FeeType, FName, MangedItem, PlayerAddr, TeamAddr, PlayerTeamAddr, BlockTime,
                    ManagedItemResponse, AssetSaleItems, AssetSaleItem, AssetSaleItemAddUpdateModel,
                    OwnershipHistory, SeasonId, MessageId, PRIOR_TO_SEASON_START_PADDING};
use shared::utils::general::{AssetTypes, GameItemTypes, generate_id_from_strings, index_string, merge_strings};
use shared::utils::general::GameItemTypes::Player;

use crate::error::ContractError;
use crate::msg::{ InstantiateMsg};
use crate::queries::{query_player_name, query_rewards_by_type_and_address,
                     query_get_all_seasons_by_league, query_check_for_season_date_range_conflicts,
                     query_get_upcoming_seasons_by_league, query_get_past_seasons_by_league,
                     query_get_active_seasons_by_league, query_get_upcoming_seasons,
                     query_get_seasons_by_season_id,
                     query_get_messages_to_item, query_get_messages_from_item, query_get_league_teams};
use crate::state::{ADMIN, HOOKS, MANAGEMENT, teams, TeamIndexes, managed_assets, PLAYER_NAMES, seasons, next_index_counter, join_season_requests, Config, season_deposits_ledger};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:goi-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //TODO: Retrieve this list from a minters contract
/*
    let config = Config {
        native_denom: msg.native_denom,
    };

    // save the owner to the INIT_CONFIG state
    INIT_CONFIG.save(deps.storage, &config)?;

 */
    let is_minter = get_minters().iter().any( |a| a == &Addr::unchecked(&info.sender.clone())  );

    match is_minter {
        true => {
            set_contract_version(deps.storage,
                                 CONTRACT_NAME, CONTRACT_VERSION)?;

            PLAYER_NAMES.save(deps.storage, &PlayerInfoPacket { items: vec![] })?;

            let save_res =
                    MANAGEMENT.save(deps.storage, &Management { fees: vec![ManagementFee{
                        id: 0,
                        created_at_block_height: _env.block.height,
                        active: true,
                        fees: Fee {
                            fee_type: FeeType::Dev,
                            description: Some("Protocol development fee(s)".to_string()),
                            to_address: _env.contract.address.clone(),
                            percent:  Decimal::from_str("0.0035").unwrap()
                        },
                    }], description: "".to_string() });

            match save_res {
                Ok(_) => {
                    match initialize_members_and_admin(deps, _env, info, Some(msg.admin.clone()), msg.members) {
                        Ok(r) => Ok(r),
                        Err(e) => {
                            Err(ContractError::GroupAdminHooksError(e))
                        }
                    }
                }
                Err(e) => {
                    Err(ContractError::Std(e))
                }
            }
        },
        false => return Err(ContractError::Unauthorized { sender: info.sender })
    }


}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        //calls to this exec path are invoked from asset contracts;
        //not call directly by user
        ExecuteMsg::AddManagedContract{ asset_name, asset_owner, contract_type } => {
            ADMIN.assert_admin(deps.as_ref(), &asset_owner).expect("Expect admin caller!");
            update_managed_status(deps.branch(), _env.block.clone(), info.sender.clone(),
                                  ManagedStatus::Enabled, asset_name.clone(),
                                  asset_owner.clone(),contract_type.clone(), true)?;
            match contract_type {
                AssetTypes::Team => {
                    teams().save(deps.storage.borrow_mut(), info.sender.clone(), &TeamInfo{
                        address: info.sender ,
                        league_assigned: None,
                        name:   match asset_name.clone() { Some(n) => n, None => "".to_string()},
                        created: BlockTime {
                            height: _env.block.height,
                            time: _env.block.time,
                            chain_id: _env.block.chain_id
                        },
                        owner: asset_owner
                    })?;
                }
                _ => {}
            }
            Ok(Response::new())
        },
        ExecuteMsg::ManagedStatusChangedHook(ManagedStatusChangedHookMsg{ change }) => {
                match is_contract_under_management(deps.storage, info.sender.clone()){
                    None => {
                        Err(ContractError::Unauthorized { sender: info.sender.clone() })
                    }
                    Some(mc) => {
                        match update_managed_status(deps, _env.block,
                                                    info.sender.clone(),
                                                    change.managed_status, mc.asset_name,
                                                    mc.asset_owner, mc.asset_type,
                                                    false) {
                            Ok(_) => Ok(Response::new()),
                            Err(e) => Err(e)
                    }
                }

                }
        }
        ExecuteMsg::GroupAdminHooks { group_admin_hooks_msg } => {
            match execute_group_admin_message(
                deps, _env, info, group_admin_hooks_msg, &ADMIN, HOOKS) {
                Ok(r) => Ok(r),
                Err(e) => Err(ContractError::GroupAdminHooksError(e))
            }
        },
        ExecuteMsg::UpdateFees { add, remove } => {
            ADMIN.assert_admin(deps.as_ref(), &info.sender).expect("User not authorized.");
            match MANAGEMENT.update_fees(deps,  info, _env, add, remove) {
                Ok(r) => Ok(r),
                Err(e) => Err(ContractError::from(e))
            }
        },
        ExecuteMsg::AddPlayersToTeam { players } => {
            process_adding_players(deps, _env, info.sender.clone(), players)
        },
        ExecuteMsg::UpdateAssetForSaleStatusHook{ for_sale_status, price } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(_) => {
                    update_asset_for_sale_status(deps, _env.block, info.sender,
                                                 for_sale_status, price)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }
        }
        ExecuteMsg::ManagedAssetSoldHook { new_owner } => {
            managed_asset_owner_changed(deps, _env.block.clone(),
                                        info.sender.clone(),
                                        new_owner.clone())

        }
        ExecuteMsg::Withdraw { recipient, amount } => {
            let is_admin_res = ADMIN.assert_admin(deps.as_ref(), &info.sender.clone());
            match is_admin_res {
                Ok(_) => {
                    let res:CosmosMsg =
                    cosmwasm_std::BankMsg::Send
                    {
                        to_address: recipient.to_string(),
                        amount
                    }.into();
                    let res_sub_msg =
                        SubMsg{
                            id: 0,
                            msg: res,
                            gas_limit: None,
                            reply_on: ReplyOn::Never
                        };
                    Ok(Response::new()
                        .add_attribute("action", "withdraw to recipient.")
                        .add_attribute("recipient", recipient)
                        //.add_attribute("amount", amount)
                        .add_submessage(res_sub_msg))
                },
                Err(_) => {
                    Err(ContractError::Unauthorized { sender: info.sender})
                },
            }

        },
        ExecuteMsg::AddSeasonToLeague { season } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(_) => {
                    add_season_to_league(deps, _env.block, info.sender, season)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }

        }
        ExecuteMsg::AddTeamsToLeague { teams, sending_user } => {
            //check if sending league is managed
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(_) => {
                    add_teams_to_league(deps, _env.block, sending_user,  info.sender, teams)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }
        }
        ExecuteMsg::UpdateSeasonStatus { season_id, status: status_update, } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(_) => {
                    update_message_status(deps, info.sender, _env.block, season_id, status_update)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }

        }
        ExecuteMsg::JoinLeague { season_id } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(mc) => {
                    join_open_season(deps, _env.block, info.sender, season_id)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }

        },
        ExecuteMsg::CancelSeasonSpot { season_id } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(mc) => {
                    cancel_team_season_spot(deps, _env.block, info.sender, season_id)
                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }
        },
        ExecuteMsg::JoinLeagueWinnerTakeAll { season_id, fee } => {
            match is_contract_under_management(deps.storage, info.sender.clone()){
                Some(mc) => {
                    let season = seasons().may_load(deps.storage,season_id.clone()).unwrap();
                    match season {
                        None => {
                            Err(ContractError::SeasonNotFound {})
                        }
                        Some(s) => {
                            join_winner_take_all_season(deps, _env.block, info.sender,
                                                        s, fee)
                        }
                    }

                },
                None =>{
                    Err(ContractError::Unauthorized{ sender: info.sender })
                },
            }

        }

    }
}





fn process_adding_players(deps: DepsMut,  _env: Env, sender: Addr, players: Vec<PlayerInfo>)
    -> Result<Response, ContractError> {
    let mut player_errors: Vec<PlayerInfo> = Vec::default();
    let mut dup_name_errors = 0;
    let mut unauthorized_request = false;
    match is_contract_under_management(deps.storage, sender.clone()) {
        None => {
            unauthorized_request = true;
        }
        Some(mc) => {
            match has_unique_player_names(players.clone()) {
                true => {

                    let mut current_state = PLAYER_NAMES.load(deps.storage)?;
                    for a_player in players.clone() {
                        let res =
                            query_player_name
                                (  match current_state.items.len() > 0
                                   { true => Some(current_state.items.clone()), false => None },
                                   a_player.first_name.clone(),
                                   a_player.last_name.clone())?;

                        match res {
                            None => {
                                current_state.items.insert( current_state.items.len(),
                                                            PlayerInfo
                                { first_name: a_player.first_name.clone(),
                                    last_name: a_player.last_name.clone(),
                                    address: a_player.address.clone(),
                                    assigned_team_address: Some(sender.clone()),
                                    position: a_player.position.clone(),
                                });
                                PLAYER_NAMES.save(deps.storage,
                                                  &current_state).expect("Failed to add player.");

                            }
                            Some(p) => {
                                match p.assigned_team_address.clone() {
                                    None => {
                                        //Do nothing. This is an unnecessary call, but Ok
                                    }
                                    Some(addr) => {
                                        match addr == sender.clone() {
                                            true => {
                                                //Do nothing. This is an unnecessary call, but Ok
                                            }
                                            false => {
                                                player_errors.push(p);

                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false => {
                    dup_name_errors += 1;
                }
            }
        }
    }
    match (player_errors.len() > 0) | (dup_name_errors > 0) | (unauthorized_request) {
        true => {
            Err(ContractError::AddPlayerErrors { players_assigned_to_another_team: player_errors,
                source_dupe_name_count: dup_name_errors,
                unauthorized_request
            })
        }
        false => {
            Ok(Response::new())
        }
    }


}

fn managed_asset_owner_changed(deps: DepsMut, block: BlockInfo,
                               sender_contract_addr: Addr, new_owner: Addr)  -> Result<Response, ContractError> {
    match is_contract_under_management(deps.storage, sender_contract_addr.clone()) {
        Some(_) => {
            let update_fn = |d: Option<MangedItem>|
                -> Result<MangedItem, ContractError>{
                let mut item = d.unwrap();
                let histor_item_count = item.ownership_history.len();
                match item.ownership_history.len() > 0 {
                    true => {
                        item.ownership_history[histor_item_count - 1].sold = Some(block.clone())
                    },
                    false => {
                        ()
                    }
                }

                Ok(
                    MangedItem {
                        asset_addr: item.asset_addr,
                        asset_name: item.asset_name,
                        asset_owner: new_owner.clone(),
                        asset_type: item.asset_type,
                        managed_status: item.managed_status,
                        created: item.created,
                        updated: block.time,
                        for_sale: 0u8,
                        for_sale_price: item.for_sale_price,
                        for_sale_price_version: item.for_sale_price_version,
                        for_sale_last_updated: item.for_sale_last_updated,
                        ownership_history: [item.ownership_history, vec![OwnershipHistory {
                            owners: new_owner,
                            purchased: block.clone(),
                            sold: None
                        }]].concat()
                    }
                )

            };

            managed_assets().update(deps.storage, &sender_contract_addr.clone(), update_fn)?;
            update_asset_for_sale_status(deps, block, sender_contract_addr, false, None )?;
            Ok(Response::new())
        },
        None => {
            Err(ContractError::Unauthorized { sender: sender_contract_addr.clone() })
        }

    }
}
fn update_managed_status(mut deps: DepsMut, block: BlockInfo, sender_contract_addr: Addr,
                         managed_status: ManagedStatus, asset_name: Option<String>,
                         asset_owner: Addr, asset_type: AssetTypes, init_creation_request: bool )
    -> Result<Response, ContractError> {

    match ( init_creation_request, managed_assets().may_load(deps.storage,
                                                             &sender_contract_addr.clone())?) {

        (false, None ) => {
            Err(ContractError::Unauthorized { sender: sender_contract_addr.clone() })
        },
        (false, Some(mc))  => {
            let update_fn = |d: Option<MangedItem>|
                -> StdResult<MangedItem>{
                match d {
                    None => {
                        Ok(MangedItem {
                            asset_addr: sender_contract_addr.clone(),
                            asset_name,
                            asset_owner: asset_owner.clone(),
                            asset_type: asset_type.clone(),
                            managed_status,
                            created: block.time,
                            updated: block.time,
                            for_sale: 0u8,
                            for_sale_price: None,
                            for_sale_price_version: 0,
                            for_sale_last_updated: Default::default(),
                            ownership_history: vec![OwnershipHistory{
                                owners: asset_owner,
                                purchased: block,
                                sold: None
                            }]
                        })
                    }
                    Some(item) => {
                        Ok(
                            MangedItem {
                                asset_addr: item.asset_addr,
                                asset_name: item.asset_name,
                                asset_owner: item.asset_owner,
                                asset_type: item.asset_type,
                                managed_status,
                                created: item.created,
                                updated: block.time,
                                for_sale: item.for_sale,
                                for_sale_price: item.for_sale_price,
                                for_sale_price_version: item.for_sale_price_version,
                                for_sale_last_updated: item.for_sale_last_updated,
                                ownership_history: item.ownership_history
                            }
                        )
                    }
                }
            };

            managed_assets().update(deps.storage, &sender_contract_addr,
                                    update_fn)?;
            Ok(Response::new())
        },
        (true, _ ) => {
            let item =
                 MangedItem {
                    asset_addr: sender_contract_addr.clone(),
                    asset_name,
                    asset_owner,
                    asset_type: asset_type.clone(),
                    managed_status,
                    created: block.time,
                    updated: block.time,
                     for_sale: 0u8,
                     for_sale_price: None,
                     for_sale_price_version: 0,
                     for_sale_last_updated: Default::default(),
                     ownership_history: vec![]
                 };
            managed_assets().save(deps.storage, &sender_contract_addr, &item)?;
            Ok(Response::new())
        }
        _ => {
                Err(ContractError::Unauthorized { sender: sender_contract_addr.clone() })
        }
    }
}

fn add_update_forsale_item(store: &mut dyn Storage, block: BlockInfo, sender: Addr,
                           update_item: AssetSaleItemAddUpdateModel) -> StdResult<()>{

    let managed_item_res = managed_assets().may_load(store, &sender.clone());
    match managed_item_res {
        Ok(r) => {
            match r {
                None => {
                    Err(StdError::not_found(sender.clone()))
                }
                Some(mut m) => {
                    m.for_sale = match update_item.for_sale { true => 1u8, false => 0u8};
                    m.for_sale_price = update_item.for_sale_price;
                    m.for_sale_price_version = update_item.for_sale_price_version;
                    m.for_sale_last_updated = block.time;
                    managed_assets().save(store, &sender.clone(), &m)?;
                    Ok(())
                }
            }
        }
        Err(e) => {
            Err(e)
        }
    }



}

fn get_managed_items_for_sale(store: &dyn Storage) -> Vec<StdResult<(Addr, MangedItem)>> {
    let res = //: Result<Vec<_>, _>
        managed_assets().idx.
            for_sale.
            sub_prefix(1u8).
            range(store, None, None, Order::Ascending).
            collect();
    res
}

fn get_forsale_items_by_asset_type(store: &dyn Storage, asset_type: AssetTypes )
    -> Option<Vec<(Addr, MangedItem)>>{
    let items_for_sale_res = get_managed_items_for_sale(store);
    match items_for_sale_res.len() > 0 {
        true => {

                    let res: Vec<(Addr, MangedItem)> =
                        items_for_sale_res.
                            into_iter().
                            filter(|mc|
                                         match mc.clone() {
                                             Ok(r) => r.1.asset_type == asset_type,
                                             Err(_) => false
                                         } ).
                            map(|i |  i.unwrap()).
                            collect();
                    Some(res)
        }
        false => {
            None
        }
    }
}

fn get_forsale_item_by_addr(store: & dyn Storage,  addr:Addr )
    -> Option<MangedItem>{
        //let res = managed_assets().may_load(store, &addr)?;
        let items_for_sale_res = get_managed_items_for_sale(store);
        match items_for_sale_res.len() > 0 {
            true => {

                let res: Vec<MangedItem> =
                    items_for_sale_res.
                        into_iter().
                        filter(|mc|
                            match mc.clone() {
                                Ok(r) => r.1.asset_addr == addr,
                                Err(_) => false
                            } ).
                        map(|i |  i.unwrap().1).
                        collect();
                Some(res[0].clone())
            }
            false => {
                None
            }
        }



}

fn is_validate_season(season: &Season, block: BlockInfo) -> Result<Response,
    ContractError>
{
    match season.validate(block) {
        true => {
            Ok(Response::new())
        }
        false => {
            Err(ContractError::InvalidSeason {})
        }
    }
}


fn verify_address_owns_assets(deps: &DepsMut, owning_address: Addr, addresses: Vec<Addr>) -> Result<bool, ContractError> {
    for item_addr in addresses {
        let res = managed_assets().may_load(deps.storage, &item_addr);
        match res {
            Ok(mi) => {
                    match mi {
                        None => {
                            return Err(ContractError::ItemNotFound { item_address: item_addr })
                        }
                        Some(m) => {
                            match m.asset_owner == owning_address {
                                true => continue,
                                false => return Ok(false)
                            }
                        }
                    }
            },
            Err(e) => {
              return  Err(ContractError::Std(e))
            }
        }       
    }
    Ok(true)
}

fn check_teams_managed_status_and_verify_asset_type(mut deps: &DepsMut, teams: Vec<TeamAddr>) -> Result<u32, ContractError>{
    let mut item: Option<MangedItem> = None;
    let mut fail_count:u32 = 0;
    for team in teams.clone() {
        item = managed_assets().may_load(deps.storage, &team)?;
        match item {
            None => {
                fail_count = fail_count + 1;
                break;
            }
            Some(a_team) => {
                match a_team.clone().asset_type == AssetTypes::Team
                    && a_team.clone().managed_status == ManagedStatus::Enabled {
                    true =>  continue,
                    false => {
                        fail_count = fail_count + 1;
                        break;
                    }

                }

            }
        }

    }
    Ok( fail_count)
}


fn verify_teams_are_available(mut deps: &DepsMut, block: BlockInfo, sending_user: Addr, sending_league_contract_addr: Addr,  team_items: Vec<TeamAddr>) -> Result<Response, ContractError> {
    let mut res:  Result<Response, ContractError> ;
    for a_team in team_items {
        let team_res = teams().may_load(deps.storage, a_team.clone())?;
        match team_res {
            None => {
                return  Err(ContractError::ItemNotFound { item_address: a_team.clone() });
            }
            Some(t) => {
                match t.league_assigned {
                    None => {
                        //None found is good, so let's
                        //continue looping through for loop
                    }
                    Some(l) => {
                        return Err(ContractError::TeamAlreadyMemberOfALeague { team_addr: a_team, league_assigned_to: l.league });
                    }
                }
            }
        }
    }
    return Ok(Response::new())
}


// Add teams to leagues
// Validation checks:
//   --check if all teams are managed
//    --make sure sending user owns teams being added
//    --check to make sure team(s) are not already assigned to a league
//    --add teams to league
///
fn add_teams_to_league(mut deps: DepsMut, block: BlockInfo, sending_user: Addr,
                       sending_league_contract_addr:
                       Addr, teams_to_assign: Vec<TeamAddr>) -> Result<Response, ContractError> {
    match check_teams_managed_status_and_verify_asset_type(deps.borrow(), teams_to_assign.clone()) {
        Ok(fv) => {
            match fv > 0 {
                true =>  Err(ContractError::InvalidTeamSubmissions{}),
                false => {
                    match verify_address_owns_assets(deps.borrow(),
                                                     sending_user.clone(),
                                                     teams_to_assign.clone()) {
                        Ok(r) => {
                            match r {
                                true => {
                                   match verify_teams_are_available(deps.borrow(), block.clone(), sending_user,
                                                                    sending_league_contract_addr.clone(),
                                                                    teams_to_assign.clone()) {
                                       Ok(_) => {
                                           for team_addr in teams_to_assign {
                                               let mut a_team  =
                                                   teams().may_load(deps.storage, team_addr.clone()).unwrap();
                                               match a_team {
                                                   None => {}
                                                   Some(mut _team) => {
                                                       _team.league_assigned =
                                                           Some(LeagueTeamAssignment{ league: sending_league_contract_addr.clone(),
                                                               assigned_date:block.time });
                                                       teams().save(deps.storage, team_addr, &_team)?;
                                                   }
                                               }
                                           }
                                           Ok(Response::new())
                                       }
                                       Err(e) => {
                                           Err(e)
                                       }
                                   }

                                },
                                false => {
                                    Err(ContractError::InvalidTeamSubmissions{})
                                }
                            }

                        }
                        Err(e) => {
                            Err(e)
                        }
                    }

                },
            }

        },
        Err(e) => {
            Err(ContractError::InvalidTeamSubmissions{})
       }
    }


}
fn get_team_season_conflicts(store: &mut dyn Storage , asset_type: AssetTypes,
                             asset_addr: Addr, start_date: Timestamp,
                             end_date: Timestamp) -> Option<Vec<Season>>{
    let sender_res: Result<Vec<_>, _> =
        join_season_requests().idx
            .sender
            .prefix((asset_type.to_u8(), asset_addr.clone()))
            .range(store, None, None, Order::Ascending)
            .collect();
    let recipient_res: Result<Vec<_>, _> =
        join_season_requests().idx
            .recipient
            .prefix((asset_type.to_u8(), asset_addr.clone()))
            .range(store, None, None, Order::Ascending)
            .collect();

    let res_items = [sender_res.unwrap(),recipient_res.unwrap() ].concat();
    let seasons: Option<Vec<Season>> =
            res_items.
            into_iter().
            filter(|i|  {
                (i.1.delivery.to.address == asset_addr || i.1.delivery.from.address == asset_addr)
                    && i.1.data.status_type == MessageTypes::Accepted {}
            }).
            map(|i|   seasons().may_load(store, i.1.data.season_id).unwrap()  ).
            filter(|season| {
                match season {
                    None => false,
                    Some(s) => {
                        (s.start_date.seconds() < start_date.seconds() && start_date.seconds() < s.end_date.seconds()) ||
                            (s.start_date.seconds() < end_date.seconds() &&  end_date.seconds() < s.end_date.seconds())
                    }
                }
            }).
            collect();

   match seasons {
       None => {
           None
       }
       Some(items) => {
           match items.len() > 0 {
               true => Some(items),
               false => None
           }
       }
   }

}


fn get_existing_join_request(store: &mut dyn Storage , season_id: SeasonId, team_addr: Addr, league_addr: Addr  ) -> Option<Message<JoinSeasonRequestInfo>>{
    let res_1: Result<Vec<_>, _> =
        join_season_requests().idx
            .season_from_to
            .prefix((season_id, team_addr.clone(), league_addr.clone()))
            .range(store, None, None, Order::Ascending)
            .collect();

    return match res_1 {
        Ok(r) => {
            match r.len() > 0 {
                true => Some(r[0].1.clone()),
                false => {
                    None
                }
            }
        }
        Err(_) => {
            None
        }
    }


}


fn get_winner_take_all_deposits_by_season(store: &dyn Storage , season_id: SeasonId) -> Vec<(u64, SeasonLedger)> {
    let res: Result<Vec<_>, _> =
        season_deposits_ledger().idx
            .season
            .prefix(season_id)
            .range(store, None, None, Order::Ascending)
            .collect();
    res.unwrap()
}


//checks if team has made a deposit for season
//if so, then distribute refund accordingly
fn distribute_individual_refund(mut deps: DepsMut, block: BlockInfo,
                                season_id: SeasonId, team_addr: TeamAddr) -> Result<Response, ContractError> {
    let a_season = seasons().may_load(deps.storage.borrow(), season_id).unwrap();

    return match a_season {
        None => {
            Err(ContractError::SeasonNotFound {})
        }
        Some(s) => {
            match s.access_type.clone() {
                None => {
                    Ok(Response::new())
                }
                Some(at) => {
                    match at.clone() {
                        SeasonAccessTypes::WinnerTakeAll { .. } => {
                            let sub_msg =
                                get_team_unpaid_refund_deposit_sub_message(deps.storage, s.clone(), team_addr.clone());

                            match sub_msg {
                                Some(sb) => {
                                    let refund_deposit = get_team_unpaid_deposit_by_season(deps.storage, s.clone(), team_addr.clone());

                                    match refund_deposit {
                                        None => Err(ContractError::SeasonDepositAlreadyClaimed {}),
                                        Some(rd) => {
                                            mark_deposit_withdrawls_paid(deps.branch(), block, vec![rd]);
                                            Ok(Response::new().add_submessage(sb))
                                        }
                                    }
                                },
                                None => Err(ContractError::SeasonDepositAlreadyClaimed {}),
                            }
                        },
                        _ => {
                            Ok(Response::new())
                        }
                    }
                }
            }
        }
    }
}

fn distribute_refunds(mut deps: DepsMut, block: BlockInfo, season_id: SeasonId) -> Result<Response, ContractError> {
    let a_season = seasons().may_load(deps.storage.borrow(), season_id).unwrap();

    return match a_season {
        None => {
            Err(ContractError::SeasonNotFound {})
        }
        Some(s) => {
            match s.access_type.clone() {
                None => {
                    Ok(Response::new())
                }
                Some(at) => {
                    match at.clone() {
                        SeasonAccessTypes::WinnerTakeAll { .. } => {
                            let sub_msgs =
                                get_unpaid_refund_deposits_submessages(deps.storage, s.clone());

                            match sub_msgs.len() > 0 {
                                true => {
                                    let refund_deposits = get_unpaid_deposits_by_season(deps.storage, s.clone());
                                    mark_deposit_withdrawls_paid(deps.branch(), block, refund_deposits);
                                    Ok(Response::new().add_submessages(sub_msgs))
                                },
                                false => Ok(Response::new())
                            }
                        },
                        _ => {
                            Ok(Response::new())
                        }
                    }
                }
            }
        }
    }
}

fn mark_deposit_withdrawls_paid(mut deps: DepsMut, block: BlockInfo, items: Vec<(u64, SeasonLedger)>) {
    for mut item in items {
        item.1.withdrawal_distribution_date = Some(block.time);
        season_deposits_ledger().save(deps.storage, item.0, &item.1).expect("Problem marking deposits as paid.");
    }
}


fn get_team_unpaid_refund_deposit_sub_message(store: &mut dyn Storage ,
                                           s: Season, team_addr: TeamAddr) ->  Option<SubMsg>{
    let unpaid_deposit =
        get_team_unpaid_deposits_by_season(store, s.clone(), team_addr.clone());
            

    match unpaid_deposit {
        None => {
            None
        }
        Some(i) => {
            let res: CosmosMsg =
                cosmwasm_std::BankMsg::Send
                {
                    to_address: i.1.team.to_string(),
                    amount: vec![i.1.team_deposit_amount]
                }.into();
            let res_sub_msg =
                SubMsg {
                    id: 0,
                    msg: res.clone(),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                };

            Some(res_sub_msg)
        }
    }

}


fn get_unpaid_refund_deposits_submessages(store: &dyn Storage , s: Season) ->  Vec<SubMsg>{
    let unpaid_deposits = get_unpaid_deposits_by_season(store, s.clone());
    let mut count: u64 = 0;
    let res: Vec<SubMsg> =
        unpaid_deposits.into_iter()
            .map(|i| {
                let res: CosmosMsg =
                    cosmwasm_std::BankMsg::Send
                    {
                        to_address: i.1.team.to_string(),
                        amount: vec![i.1.team_deposit_amount]
                    }.into();
                let res_sub_msg =
                    SubMsg {
                        id: count,
                        msg: res,
                        gas_limit: None,
                        reply_on: ReplyOn::Never
                    };
                count += 1;
                res_sub_msg
            })
            .collect();
    res
}


fn get_team_unpaid_deposit_by_season(store: &dyn Storage , s: Season, team_addr: TeamAddr) -> Option<(u64, SeasonLedger)> {
    let unpaid_deposit: Vec<(u64, SeasonLedger)> =
        get_winner_take_all_deposits_by_season(store, s.id).
            into_iter().
            filter(|item| item.1.withdrawal_distribution_date.is_none() && item.1.team == team_addr).
            collect();
    match unpaid_deposit.len() > 0 {
        true => {
            Some(unpaid_deposit[0].clone())
        },
        false => {
            None

        }
    }
}

fn get_team_unpaid_deposits_by_season(store: &dyn Storage , s: Season, team_addr: TeamAddr) -> Option<(u64, SeasonLedger)> {
    let unpaid_deposit: Vec<(u64, SeasonLedger)> =
        get_winner_take_all_deposits_by_season(store, s.id).
            into_iter().
            filter(|item| item.1.withdrawal_distribution_date.is_none() && item.1.team == team_addr).
            collect();
    match unpaid_deposit.len() > 0 {
        true => {
            Some(unpaid_deposit[0].clone())
        },
        false => {
            None
        }
    }
}

fn get_unpaid_deposits_by_season(store: &dyn Storage , s: Season) -> Vec<(u64, SeasonLedger)> {
    let unpaid_deposits: Vec<(u64, SeasonLedger)> =
        get_winner_take_all_deposits_by_season(store, s.id).
            into_iter().
            filter(|item| item.1.withdrawal_distribution_date.is_none()).
            collect();
    unpaid_deposits
}

fn update_message_status(mut deps: DepsMut, sending_league: Addr, block: BlockInfo,
     season_id: SeasonId, update_status_type: MessageTypes) -> Result<Response, ContractError> {
    let a_season = seasons().may_load(deps.storage.borrow(), season_id).unwrap();

    //let invite_res = join_season_requests().may_load(deps.storage, season_id)?;
    match a_season {
        None => {
            Err(ContractError::SeasonNotFound{  })
        }
        Some(mut se) => {
            return match update_status_type {
                MessageTypes::CancelSeason {} => {
                    //make sure league sending update own season being updated
                    match se.league == sending_league {
                        true => {
                            match (se.start_date < block.time, (get_season_teams_accepted_count(deps.storage, se.id.clone())) > 1) {
                                (true, true) => {
                                    Err(ContractError::TooLateToCancelSeason {})
                                },
                                (true, false)/* not enough teams accepted to start season, so allow cancel */ |
                                (false, _) /* season hasn't started, so allow cancel */ => {
                                    se.status = Some(SeasonStatus::Cancelled { date_cancelled: block.clone() });
                                    seasons().save(deps.storage, se.id, &se)?;
                                    let res = distribute_refunds(deps, block.clone(), se.id);
                                    match res {
                                        Ok(r) => {
                                            //response from distribute_refunds contains sub_messages
                                            //needed for payment transfers
                                            Ok(r)
                                        }
                                        Err(e) => {
                                            Err(e)
                                        }
                                    }
                                }
                            }
                        }
                        false => {
                            Err(ContractError::Unauthorized { sender: sending_league })
                        }
                    }
                }
                _ => {
                    Err(ContractError::Unauthorized { sender: sending_league })
                }
            }


        }
    }
}



fn process_season_join_request(mut deps: DepsMut, block: BlockInfo, season: Season,
                               team_addr: Addr) -> StdResult<()> {


            let id = next_index_counter(deps.storage)?;
            join_season_requests().save(deps.storage, id, &Message {
                id,
                updated: block.time.clone(),
                created: block.time.clone(),
                delivery:  DeliveryInfo {
                        to: DeliveryPacket {
                            asset_type: AssetTypes::League,
                            address: season.league.clone()
                        },
                        from: DeliveryPacket {
                            asset_type: AssetTypes::Team,
                            address: team_addr.clone()
                        }

                },
                data: JoinSeasonRequestInfo {
                    status_type: MessageTypes::Accepted {},
                    season_id: season.id
                },
                notes: vec![]
            })

}




fn join_winner_take_all_season(mut deps: DepsMut, block: BlockInfo, team_addr: Addr,
                               season: Season, funds: Vec<Coin>) -> Result<Response, ContractError> {
   let res =  join_season(deps.branch(), block.clone(), team_addr.clone(), season.id.clone(), Some(funds.clone()));
    match res {
        Ok(r) => {
            //TODO: add sanity check on funds balance vs what's
            //been sent in for kitties and payouts all balances out

            let id = next_index_counter( deps.storage)?;
            season_deposits_ledger().save(deps.storage, id, &SeasonLedger {
                id,
                season_id: season.id.clone(),
                league: season.league,
                team: team_addr,
                team_deposit_amount: funds[0].clone(),
                deposit_date: block.time.clone(),
                withdrawal_distribution_date: None
            }).expect("Problem saving season deposit.");
            Ok(r)
        }
        Err(e) => {
            Err(e)
        }
    }
}


fn join_open_season(mut deps: DepsMut, block: BlockInfo, team_addr: Addr,
                    season_id: SeasonId) -> Result<Response, ContractError> {
    join_season(deps, block, team_addr, season_id, None)

}

fn get_season_league_entry_for_team(store: &dyn Storage, team_addr: Addr,
                                    season_id: SeasonId, league_addr: Addr) -> Option<(u64, Message<JoinSeasonRequestInfo>)> {
    let res: Result<Vec<_>, _> =
        join_season_requests().idx
            .season_from_to
            .prefix((season_id, team_addr,league_addr))
            .range(store, None, None, Order::Ascending)
            .collect();

    match res {
        Ok(r ) => {
            match r.len() > 0 {
                true =>{
                    Some(r[0].clone())
                },
                false => None
            }

        }
        Err(_) => {
            None
        }
    }

}

fn cancel_team_season_spot(deps: DepsMut, block: BlockInfo, team_addr: Addr,
                           season_id: SeasonId) -> Result<Response, ContractError> {
    let season = seasons().may_load(deps.storage,season_id.clone()).unwrap();
    return match season {
        None => {
            Err(ContractError::SeasonNotFound {})
        },
        Some(se) => {
            match (se.start_date < block.time, (get_season_teams_accepted_count(deps.storage, se.id.clone())) > 1) {
                (true, true) => {
                    Err(ContractError::TooLateToCancelSeason {})
                },
                (true, false)/* not enough teams accepted to start season, so allow cancel */ |
                (false, _) /* season hasn't started, so allow cancel */ => {
                    let team_entry = get_season_league_entry_for_team(deps.storage, team_addr.clone(), se.id, se.league);
                    if let Some(mut te) = team_entry {
                        let mut item = te.clone().1;
                        item.data.status_type = MessageTypes::CancelSeason {};
                        join_season_requests().save(deps.storage, te.0, &item).expect("Problem processing cancel request.");


                        let res = distribute_individual_refund(deps, block.clone(), se.id, team_addr);
                        match res {
                            Ok(r) => {
                                //response from distribute_refund contains sub_message
                                //needed for payment transfers
                                Ok(r)
                            }
                            Err(e) => {
                                Err(e)
                            }
                        }
                    } else {
                        Err(ContractError::TeamNotMemberOfSeason {})
                    }
                }
            }
        }
    }

}

fn join_season(mut deps: DepsMut, block: BlockInfo, team_addr: Addr,
               season_id: SeasonId, funds: Option<Vec<Coin>>) -> Result<Response, ContractError>  {
    let season = seasons().may_load(deps.storage,season_id.clone()).unwrap();
    match season {
        None => {
            Err(ContractError::SeasonNotFound {})
        }
        Some(s) => {
            let validation_res =
                validate_season_join_request(deps.storage, block.clone(), team_addr.clone(), s.league.clone(), funds, s.clone() );

            match validation_res {
                Ok(r) => {
                    let process_res = process_season_join_request(deps.branch(), block, s.clone(),
                                                                  team_addr.clone());
                    match process_res {
                        Ok(_) => {
                            Ok(r)
                        },
                        Err(e) => {
                            Err(ContractError::Std(e))
                        }
                    }
                },
                Err(e) => Err(e)
            }
        }
    }


}


fn get_season_teams_accepted_count (store: & dyn Storage , season_id: SeasonId) -> u32 {
    let res: Result<Vec<_>, _> =
        join_season_requests().idx
            .season_id
            .prefix(season_id)
            .range(store, None, None, Order::Ascending)
            .filter(|s|
                {
                    match s {
                        Ok(item) => {
                            item.1.data.status_type == MessageTypes::Accepted {}
                        },
                        _ => false,
                    }

                })
            .collect();

    match res {
        Ok(r) => {
            
            r.len() as u32
        },
        _ => panic!("Problem calculating teams accepted count!")

    }
}


fn is_season_at_capacity (store: &mut dyn Storage , season_id: SeasonId) -> bool {
    let res_1: Result<Vec<_>, _> =
        join_season_requests().idx
            .season_id
            .prefix(season_id)
            .range(store, None, None, Order::Ascending)
            .filter(|s|
                {
                    match s {
                        Ok(item) => {
                            item.1.data.status_type == MessageTypes::Accepted {}
                        },
                        _ => false,
                    }
                 
                })
            .collect();

    match res_1 {
        Ok(r) => {
            let season = seasons().may_load(store ,season_id.clone()).unwrap().unwrap();
            !((r.len() as u32) < season.max_teams_allowed.unwrap())
        },
        _ => true

    }
}

fn validate_season_join_request(store: &mut dyn Storage , block: BlockInfo, team_addr: Addr,
                                league_addr: Addr, funds: Option<Vec<Coin>>, season: Season)
    -> Result<Response, ContractError> {


            //(15 mins) padding time to give
            //some time for teams response
            //PRIOR_TO_SEASON_START_PADDING = 900
            return match season.start_date.seconds() - block.time.seconds() >= PRIOR_TO_SEASON_START_PADDING {
                true => {
                    match season.status {
                        None => {
                            return Err(ContractError::SeasonStatusNotSet {})
                        },
                        Some(cs) => {
                            match cs {
                                SeasonStatus::Active => {

                                    match is_season_at_capacity(store, season.id.clone()) {
                                        true => {
                                            return Err(ContractError::SeasonHasReachedCapacity {})
                                        }
                                        false => {
                                            //Ok, allow to continue
                                        }
                                    }
                                },
                                SeasonStatus::Cancelled { date_cancelled } => {
                                    return Err(ContractError::SeasonStatusCancelled { date_cancelled})
                                }
                                SeasonStatus::Private => {
                                    return Err(ContractError::SeasonStatusPrivate {})
                                }
                            }
                        }
                    }

                    let existing_request_res =
                            get_existing_join_request(store, season.id.clone()
                                                      , team_addr.clone(), league_addr);
                    match existing_request_res {
                        None => {
                            //Ok, allow to continue
                        }
                        Some(_) => {
                            return Err(ContractError::TeamAlreadyMemberOfSeason {})
                        }
                    }
                    match season.access_type.clone() {
                        Some(st) => {
                            match st {
                                SeasonAccessTypes::Open => {
                                    //Ok, allow to continue
                                }
                                SeasonAccessTypes::WinnerTakeAll { coin } => {

                                    match funds {
                                        None => {
                                            return Err(ContractError::IncorrectFundingSent {})
                                        }
                                        Some(f) => {
                                            match coin.amount == f[0].amount &&
                                                coin.denom == f[0].denom {
                                                true => {
                                                    //Ok, allow to continue
                                                }
                                                false => {
                                                    return Err(ContractError::IncorrectFundingSent {})
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            return Err(ContractError::SeasonTypeNotSet {})
                        }
                    }

                    let conflicting_seasons =
                        get_team_season_conflicts(store, AssetTypes::Team,
                                                  team_addr.clone(),
                                                  season.start_date.clone(),
                                                  season.end_date.clone());
                    match conflicting_seasons {
                        None => {
                            Ok(Response::new())
                        }
                        Some(items) => {
                            let season_ids: Vec<SeasonId> =
                                items.
                                    into_iter().
                                    map(|s| s.id).
                                    collect();
                            Err(ContractError::SeasonScheduleConflict { conflicting_season: season_ids })
                        }
                    }
                }
                false => {
                    Err(ContractError::TooLateToRequestToJoinLeagueSeason {})
                }
            }

}

fn add_season_to_league(mut deps: DepsMut, block: BlockInfo,
                        sender_contract_addr: Addr, mut season: Season) -> Result<Response,
    ContractError>{
    
    match is_validate_season(season.borrow(), block.clone()) {
        Ok(r) => {

            match managed_assets().may_load(deps.storage, &sender_contract_addr.clone())? {
                None => {
                    Err(ContractError::Unauthorized { sender: sender_contract_addr.clone() })
                },
                Some(mut mc) => {
                    match mc.clone().asset_type == AssetTypes::League {
                        true => {
                            match query_check_for_season_date_range_conflicts(deps.storage,
                                                                              season.clone().start_date.seconds(),
                                                                  season.clone().end_date.seconds(),
                                                                  sender_contract_addr.clone()){
                                Ok(res) => {
                                    match res {
                                        None => {

                                            let id = next_index_counter( deps.storage)?;
                                            season.id = id.clone();
                                            //let's make sure correct league aggress is assigned to
                                            //new season
                                            season.league = mc.asset_addr;
                                            seasons().save(deps.storage, id , &season)?;
                                            Ok(Response::new())
                                        }
                                        Some(items) => {
                                            let seasons: Vec<SeasonId> =
                                                    items.
                                                        into_iter().
                                                        map( |s| s.id).
                                                        collect();
                                            Err(ContractError::SeasonScheduleConflict{ conflicting_season: seasons })
                                        }
                                    }
                                },
                                Err(e) => {
                                    Err(ContractError::Std(e))
                                }
                            }

                        }
                        false => {
                            Err(ContractError::Unauthorized { sender: sender_contract_addr })
                        }
                    }

                }
            }
        },
        Err(e) => {
            Err(e)
        }
    }

}

fn update_asset_for_sale_status(deps: DepsMut, block: BlockInfo, sender_contract_addr: Addr,
                                for_sale_status: bool, price: Option<Coin>) -> Result<Response,
    ContractError> {
    match managed_assets().may_load(deps.storage, &sender_contract_addr.clone())? {
        None => {
            Err(ContractError::Unauthorized { sender: sender_contract_addr.clone() })
        },
        Some(mut mc) => {
                mc.for_sale_last_updated = block.time;
                mc.for_sale = match for_sale_status { true => 1u8, false => 0u8};
                mc.for_sale_price = match price { Some(p) => Some(p), None =>  price};
                mc.updated = block.time;
                mc.for_sale_price_version = mc.for_sale_price_version + 1;
                managed_assets().save(deps.storage,&sender_contract_addr.clone(), &mc)?;
            Ok(Response::new())
        }
    }
}

fn add_team_player_to_team(deps: DepsMut, block: BlockInfo, team_sender_contract_addr: TeamAddr, player_info:PlayerInfo) -> StdResult<()> {
    let res = managed_assets().may_load(deps.storage, &team_sender_contract_addr.clone())?;
    match res {
        Some(ma) => {
            match ma.managed_status == ManagedStatus::Enabled  {
                true => {

                    let team_name = match ma.asset_name {
                        None => "".to_string(),
                        Some(n) => n
                    };
                    teams().save(deps.storage, team_sender_contract_addr.clone(),
                                 &TeamInfo::new(None,team_sender_contract_addr.clone(),
                                                team_name.clone(), player_info,
                                                ma.asset_owner.clone(), BlockTime {
                        height:block.height,
                        time: block.time,
                        chain_id: block.chain_id
                    }))
                }
                false => {
                    Err(StdError::not_found(team_sender_contract_addr.clone()))
                }
            }

        }
        None => {
            Err(StdError::not_found(team_sender_contract_addr.clone()))
        }

    }
}

fn is_contract_under_management(store: &mut dyn Storage , sender: Addr) -> Option<MangedItem> {
    let res   =  managed_assets().may_load(store, &sender);

    match res  {
        Ok(r) => {
            match r  {
                None => None,
                Some(mc) => {
                    match mc.managed_status == ManagedStatus::Enabled  {
                        true => Some(mc),
                        false => None,
                    }

                }
            }
        }
        Err(_) => {
            None
        }
    }


}

fn no_player () -> StdResult<Option<PlayerInfo>> {
     Ok(None)
}

fn no_managed_contract_info_response () -> StdResult<Option<ManagedContractInfoResponse>> {
    Ok(None)
}


//make sure there are no dupes in collection of players sent
fn has_unique_player_names(iter: Vec<PlayerInfo>) -> bool
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x|
        {
            let key:String = x.first_name.trim().to_lowercase() + &x.last_name.trim().to_lowercase();
            uniq.insert(key)
        })
}



fn validate_player_names(store: &mut dyn Storage , players: Vec<PlayerInfo>) -> Result<(), ContractError>{
    match has_unique_player_names(players.clone()) {
        true => {
            let mut count = 0;
            for a_player in players.clone(){
                match is_name_in_use(store, a_player.first_name, a_player.last_name){
                    true => {
                        count += 1;
                    }
                    _ => {}
                }
            }

            match count > 0 {
                true => {
                    Err(ContractError::AddPlayerErrors {
                        players_assigned_to_another_team: vec![],
                        source_dupe_name_count: count,
                        unauthorized_request: false
                    })
                }
                false => {
                    Ok(())
                }
            }
        }
        false => {

            Err(ContractError::AddPlayerErrors {
                players_assigned_to_another_team: vec![],
                source_dupe_name_count: 1,
                unauthorized_request: false
            })
        }
    }
}


fn is_name_in_use(store: &mut dyn Storage, first_name: String, last_name: String) -> bool{
    let current_state_res = PLAYER_NAMES.load(store);
    match current_state_res {
        Ok(p) => {
            let res:  Vec<PlayerInfo> =
                p.items
                    .into_iter()
                    .filter(| i | i.last_name.to_lowercase().trim() == last_name.to_lowercase().trim() &&
                                             i.first_name.to_lowercase().trim() == first_name.to_lowercase().trim())
                    .collect();
            match res.len() > 0 {
                true => true,
                false => false
            }
        }
        Err(_) => false
    }
}

pub fn process_managed_asset_items_for_response(items:Vec<(Addr, MangedItem)>)
                                                -> Option<Vec<ManagedItemResponse>>{
    match items.len() > 0 {
        true => {
            let mut managed_response_items: Vec<ManagedItemResponse> = Default::default();
            for item in items.clone() {
                managed_response_items.push(ManagedItemResponse {
                    managed_status: item.1.managed_status,
                    for_sale: match item.1.for_sale { 1u8 => true, _ => false},
                    asset_type: item.1.asset_type,
                    contract_addr: Some(item.0),
                    for_sale_price: item.1.for_sale_price,
                    for_sale_price_version: item.1.for_sale_price_version,
                    for_sale_last_updated: item.1.for_sale_last_updated,
                    ownership_history: item.1.ownership_history
                });
            }
            Some(managed_response_items)
        },
        false => {
            None
        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: GoiManagerQueryMsg) -> StdResult<Binary> {

    match msg {
        GoiManagerQueryMsg::ManagementQryMessages { management_qry_msg } => {
            match management_qry_msg {
                ManagementQryMsg::GetManagementInfo {} => to_binary(&query_management_info(deps, MANAGEMENT)?),
                ManagementQryMsg::GetManagedContract { contract } => {
                    match managed_assets().may_load(deps.storage, &contract.clone()) {
                        Ok(r) => {
                            match r {
                                Some(ma) => {
                                    let res:Option<ManagedContractInfoResponse> =

                                            Some(ManagedContractInfoResponse {
                                                contract: Some(ManagedContract {
                                                    managed: ma.managed_status,
                                                    contract,
                                                    contract_type: ma.asset_type
                                                })
                                            });
                                    to_binary(&res)
                                },
                                None => { to_binary(&no_managed_contract_info_response().unwrap()) }
                            }
                        },
                        _ => {
                            to_binary(&no_managed_contract_info_response().unwrap())
                        }
                    }
                }
            }
        },
        GoiManagerQueryMsg::GetManagedContract { contract_address, contract_type } => {
            let res =
                match managed_assets().may_load(deps.storage, &contract_address)? {
                    None => None,
                    Some(mc) => {

                        Some(ManagedItemResponse {
                            managed_status: mc.managed_status,
                            for_sale:  match mc.for_sale { 1u8 => true, _ => false},
                            asset_type: mc.asset_type,
                            contract_addr: Some (mc.asset_addr),
                            for_sale_price: mc.for_sale_price,
                            for_sale_price_version: mc.for_sale_price_version,
                            for_sale_last_updated: mc.for_sale_last_updated,
                            ownership_history: mc.ownership_history
                        })
                    }
                };
            to_binary(&res)
        },
        GoiManagerQueryMsg::GetPlayerByName { first_name: f_name, last_name: l_name } => {
            let current_state = PLAYER_NAMES.load(deps.storage)?;
            match current_state.items.len() > 0 {
                false => {
                    to_binary(&no_player().unwrap())
                },
                true => {

                    to_binary( &query_player_name
                        ( Some(current_state.items), f_name, l_name)?)
                }
            }
        },
        GoiManagerQueryMsg::GetOwnerAssets { owner_address } => {
            let res: Result<Vec<_>, _> =
                managed_assets().idx
                    .owner
                    .prefix(owner_address)
                    .range(deps.storage, None, None, Order::Ascending)
                    .collect();
            let res_item =
                process_managed_asset_items_for_response(res?);
            to_binary(&res_item)
        }
        GoiManagerQueryMsg::GetAssetsForSale { contract_type } => {
            let asset_items = get_forsale_items_by_asset_type(deps.storage, contract_type);
            let res =
                match asset_items {
                    Some(items) => {
                        process_managed_asset_items_for_response(items)
                    },
                    None => {
                        None
                    }
                };
            to_binary(&res)
        }
        GoiManagerQueryMsg::GetAllSeasonsForLeague { league_address } => {
          let res = query_get_all_seasons_by_league(deps.storage, league_address);
          to_binary(&res)
        }
        GoiManagerQueryMsg::GetUpcomingSeasonsForLeague { league_address } => {
            let res = query_get_upcoming_seasons_by_league(deps.storage, league_address, _env.block);
            to_binary(&res)
        }
        GoiManagerQueryMsg::GetPastSeasonsForLeague { league_address } => {
            let res = query_get_past_seasons_by_league(deps.storage, league_address, _env.block);
            to_binary(&res)
        }
        GoiManagerQueryMsg::GetSeasonById { season_id } => {
            let res = query_get_seasons_by_season_id(deps.storage, season_id , _env.block);
            to_binary(&res)
        }
        GoiManagerQueryMsg::GetUpComingSeasonsForAllLeagues { } => {
            let res = query_get_upcoming_seasons(deps.storage, _env.block);
            to_binary(&res)
        }
        GoiManagerQueryMsg::CheckSeasonDateRangeForLeague { start_date, end_date, league_addr } => {
            let res = query_check_for_season_date_range_conflicts(deps.storage, start_date.seconds(), end_date.seconds(), league_addr)?;
            to_binary(&res)
        }
        GoiManagerQueryMsg::GetActiveSeasonsForLeague { league_address } => {
            let res = query_get_active_seasons_by_league(deps.storage, league_address, _env.block);
            to_binary(&res)
        },
        GoiManagerQueryMsg::GetMessagesToItem { item_addr, asset_type } => {
            let res = query_get_messages_to_item(deps.storage, item_addr, asset_type);
            to_binary(&res)
        },
        GoiManagerQueryMsg::GetMessagesFromItem { item_addr, asset_type } => {
            let res = query_get_messages_from_item(deps.storage, item_addr, asset_type);
            to_binary(&res)
        },
        GoiManagerQueryMsg::GetLeagueTeams { league_addr } => {
            let res = query_get_league_teams(deps.storage, league_addr);
            to_binary(&res)
        }

    }
}



