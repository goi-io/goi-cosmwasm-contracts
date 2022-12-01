use std::collections::HashMap;

use cosmwasm_std::{Addr, BlockInfo, Order, StdError, StdResult, Storage};
use cosmwasm_std::Order::Ascending;
use shared::messaging::{JoinSeasonRequestInfo, Message};


use shared::player::PlayerInfo;
use shared::rewards::{Reward, RewardTypes};
use shared::season::Season;
use shared::team::TeamInfo;
use shared::utils::{AsseTypes_u8, LeagueAddr};
use shared::utils::general::AssetTypes;

use crate::state::{join_season_requests, seasons, teams};

pub fn query_player_name
(player_items: Option<Vec<PlayerInfo>>, first_name: String,
 last_name: String) -> StdResult<Option<PlayerInfo>> {

    match player_items {
        None => Ok(None),
        Some(pls) => {
            let p_res: Vec<PlayerInfo> =
                pls.into_iter().filter(|p|
                    p.first_name == first_name &&
                    p.last_name == last_name)
                    .collect();
            match p_res.len() > 0 {
                false => { Ok(None)}
                true => {
                    Ok(Some(p_res[0].clone()))
                }
            }
        }
    }
}




pub fn query_rewards_by_type_and_address<'a>(state_rewards: &Option<HashMap<Addr, Vec<Reward>>>,
                                             search_address: Addr, reward_type: RewardTypes)
                                             -> Option<Vec<&Reward>> {
    match state_rewards {
        None => {
            None
        },
        Some(rs) => {
           match rs.get(&search_address) {
               None => {
                   None
               }
               Some(r) => {
                  let hold =
                   r
                   .into_iter()
                   .filter(|i| i.reward_type == reward_type)
                   .collect::<Vec<&Reward>>();

                  Some( hold )
               }
           }
        }
    }
}



pub fn query_get_upcoming_seasons(store: &dyn Storage, block: BlockInfo) -> Option<Vec<Season>>  {
    let res =
        seasons().idx.
            owning_league.
            //prefix_range(&store,  Some(PrefixBound::exclusive(block.time.seconds()))  , None, Order::Ascending).
            prefix_range(store, None, None, Ascending).
            filter(| l | {
                    match l {
                        Ok(r) => {
                            r.1.start_date.seconds() > block.time.seconds()
                        },
                        _  => false
                    }
            }).
            collect::<StdResult<Vec<_>>>();
    into_seasons_vec(res)
}


pub fn query_get_seasons_by_season_id(store: &dyn Storage, season_id: u64, block: BlockInfo) -> Option<Season>  {
    seasons().may_load(store, season_id).unwrap()
}


pub fn query_get_upcoming_seasons_by_league(store: &dyn Storage, league_addr: Addr, block: BlockInfo) -> Option<Vec<Season>>  {
    let res =
        seasons().idx.
            owning_league.
            //prefix_range(&store,  Some(PrefixBound::exclusive(block.time.seconds()))  , None, Order::Ascending).
            prefix(league_addr).
            range(store, None, None, Ascending).
            filter(| l | { 
                        match l {  Ok(r) =>  r.1.start_date.seconds() > block.time.seconds(), _ => false}

                }).
            collect::<StdResult<Vec<_>>>();
    into_seasons_vec(res)
}



pub fn query_get_messages_to_item(store: &dyn Storage, item_addr: Addr, item_asset_type: AssetTypes) -> Option<Vec<Message<JoinSeasonRequestInfo>>>  {
    let res=
        join_season_requests().idx.
            recipient.
            prefix((item_asset_type.to_u8(), item_addr)).
            range(store, None, None, Ascending).
            collect();
    into_invite_vec(res)
}

pub fn query_get_messages_from_item(store: &dyn Storage, item_addr: Addr, item_asset_type: AssetTypes) -> Option<Vec<Message<JoinSeasonRequestInfo>>>  {
    let res: Result<Vec<_>, _> =
        join_season_requests().idx.
            sender.
            prefix((item_asset_type.to_u8(), item_addr)).
            range(store, None, None, Ascending).
            collect();
    into_invite_vec(res)
}


pub fn query_get_league_teams(store: &dyn Storage, league_addr: LeagueAddr) -> Option<Vec<TeamInfo>>  {
    let res: Result<Vec<_>, _> =
        teams().idx.
            leagues.
            prefix(league_addr).
            range(store, None, None, Ascending).
            collect();
    into_teams_vec(res)
}

pub fn query_get_active_seasons_by_league(store: &dyn Storage, league_addr: Addr, block: BlockInfo) -> Option<Vec<Season>>  {
    let res: Result<Vec<_>, _> =
        seasons().idx.
            owning_league.
            //prefix_range(&store,  Some(PrefixBound::exclusive(block.time.seconds()))  , None, Order::Ascending).
            prefix(league_addr).
            range(store, None, None, Ascending).
            filter(| l |

                match l {
                    Ok(r) =>
                        {
                            let test = r.1.clone();
                            (r.1.end_date.seconds() > block.time.seconds() &&
                                r.1.start_date.seconds() < block.time.seconds())
                        },
                    _ => false
                }).
            collect();
    into_seasons_vec(res)
}


pub fn query_get_past_seasons_by_league(store: &dyn Storage, league_addr: Addr, block: BlockInfo) -> Option<Vec<Season>>  {
    let res: Result<Vec<_>, _> =
        seasons().idx.
            owning_league.
            //prefix_range(&store,  Some(PrefixBound::exclusive(block.time.seconds()))  , None, Order::Ascending).
            prefix(league_addr).
            range(store, None, None, Ascending).
            filter(| l | 
                
                match l {
                     Ok(r) => r.1.end_date.seconds() < block.time.seconds(),
                     _ => false
                    }).
            collect();
    into_seasons_vec(res)
}

fn into_seasons_vec(res: Result<Vec<(u64, Season)>, StdError>) -> Option<Vec<Season>> {
    match res {
        Ok(items) => {
            match items.len() > 0 {
                true => {
                    let items_res: Vec<Season> =
                        items.
                            into_iter().
                            map(|i| i.1).
                            collect();
                    Some(items_res)
                },
                false => None
            }
        }
        Err(e) => {
            None
        }
    }
}



fn into_teams_vec(res: Result<Vec<(Addr, TeamInfo)>, StdError>) -> Option<Vec<TeamInfo>> {
    match res {
        Ok(items) => {
            match items.len() > 0 {
                true => {
                    let items_res: Vec<TeamInfo> =
                        items.
                            into_iter().
                            map(|i| i.1).
                            collect();
                    Some(items_res)
                },
                false => None
            }
        }
        Err(e) => {
            None
        }
    }
}







fn into_invite_vec(res: Result<Vec<(u64, Message<JoinSeasonRequestInfo>)>, StdError>) -> Option<Vec<Message<JoinSeasonRequestInfo>>> {
    match res {
        Ok(items) => {
            match items.len() > 0 {
                true => {
                    let items_res: Vec<Message<JoinSeasonRequestInfo>> =
                        items.
                            into_iter().
                            map(|i| i.1).
                            collect();
                    Some(items_res)
                },
                false => None
            }
        }
        Err(e) => {
            None
        }
    }
}


pub fn query_get_all_seasons_by_league(store: &dyn Storage, league_addr: Addr) -> Option<Vec<Season>>  {
    let res =
        seasons().idx.
            owning_league.
            //prefix_range(&store,  Some(PrefixBound::exclusive(block.time.seconds()))  , None, Order::Ascending).
            prefix(league_addr).
            range(store, None, None, Ascending).
            collect::<StdResult<Vec<_>>>();
    into_seasons_vec(res)
}


pub fn query_check_for_season_date_range_conflicts(store: &dyn Storage, start_date: u64, end_date: u64, league_addr: Addr) -> StdResult<Option<Vec<Season>>>  {
    let res =
        seasons().idx.
            owning_league.
            prefix(league_addr).
            range(store, None, None, Order::Ascending).
            filter(|res |
                {
                     match res {
                        Ok(r) => {
                            let (_, a_season) = r;
                            (a_season.start_date.seconds() < start_date && start_date < a_season.end_date.seconds()) ||
                                (a_season.start_date.seconds() < end_date &&  end_date < a_season.end_date.seconds())
                        },
                        Err(_) => false
                     }

                }).
            collect::<StdResult<Vec<_>>>();
    match res {
        Ok(items) => {
            match items.len() > 0 {
                true => {
                    let items_res: Vec<Season> =
                        items.
                            into_iter().
                            map(|i| i.1).
                            collect();
                    Ok(Some(items_res))
                },
                false => Ok(None)
            }
        }
        Err(e) => {
            Err(e)
        }
    }
}


