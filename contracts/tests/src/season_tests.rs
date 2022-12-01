use std::borrow::BorrowMut;
use anyhow::Error;

use cosmwasm_std::{Addr, Coin, Uint128};
use cosmwasm_std::testing::{mock_env, MockStorage};
use cw_multi_test::{App, AppResponse, Executor};
use goi_manager::ContractError;
use shared::messaging::MessageTypes;
use shared::season::SeasonAccessTypes;
use shared::utils::{PRIOR_TO_SEASON_START_PADDING, THIRTY_MINUTES, FIFTEEN_MINUTES, ONE_MINUTE};
use shared::utils::general::AssetTypes;
use team::msg::ExecuteMsg;
use crate::shared_utils::{get_league_for_users, get_season, get_team_for_users, instantiate_management_contract, mock_app_by_user, OWNER, TOKEN, USER1, USER2, USER3, add_season_to_league, team_request_to_join_league, update_message_status, get_messages, get_season_with_custom_settings};

#[test]
fn league_season_scheduling_conflicts() {
    let mut season_id = 1u64;
    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1, USER2, USER3], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let block_time =  mock_env().block.time.clone();

    let goi_manager_addr = instantiate_management_contract(&mut app);

    let user1_league_addr = get_league_for_users(vec![Addr::unchecked(USER1)],
                                                 goi_manager_addr.clone(), app.borrow_mut());


    let season_1_start_date = block_time.plus_seconds( PRIOR_TO_SEASON_START_PADDING + 300);
    let season_1_end_date = season_1_start_date.plus_seconds(THIRTY_MINUTES);
    let season_1 = get_season(season_id, season_1_start_date.clone(), season_1_end_date.clone());
    season_id = season_id + 1u64;


    let season_2_start_date = season_1_start_date.plus_seconds(FIFTEEN_MINUTES);
    let season_2_end_date = season_1_end_date.plus_seconds(FIFTEEN_MINUTES);
    let season_2 = get_season(season_id, season_2_start_date, season_2_end_date);
    season_id = season_id + 1u64;


    let add_season_to_league_res1 = add_season_to_league(app.borrow_mut(),
                                              USER1,  season_1, user1_league_addr[0].clone());

    let add_season_to_league_res2 = add_season_to_league(app.borrow_mut(),
                                              USER1,  season_2, user1_league_addr[0].clone());

    match add_season_to_league_res2 {
        Ok(_) => {
            assert!(false)
        }
        Err(e) => {
            assert_eq!(goi_manager::ContractError::SeasonScheduleConflict { conflicting_season: vec![1] }, e.downcast::<ContractError>().unwrap())
        }
    }

}



#[test]
fn team_season_join_conflicts() {

    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1, USER2, USER3], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let block_time =  mock_env().block.time.clone();


    let goi_manager_addr = instantiate_management_contract(&mut app);

    let user1_team_addr = get_team_for_users(vec![Addr::unchecked(USER1)],
                                             goi_manager_addr.clone(), app.borrow_mut());
    let user2_team_addr = get_team_for_users(vec![Addr::unchecked(USER2)],
                                             goi_manager_addr.clone(), app.borrow_mut());
    let user3_team_addr = get_team_for_users(vec![Addr::unchecked(USER3)],
                                             goi_manager_addr.clone(), app.borrow_mut());



    let user1_league_addr = get_league_for_users(vec![Addr::unchecked(USER1)],
                                                 goi_manager_addr.clone(), app.borrow_mut());


    let user2_league_addr = get_league_for_users(vec![Addr::unchecked(USER2)],
                                                 goi_manager_addr.clone(), app.borrow_mut());

    let user3_league_addr = get_league_for_users(vec![Addr::unchecked(USER3)],
                                                 goi_manager_addr.clone(), app.borrow_mut());



    let mut season_id = 1u64;
    let season_1_start_date = block_time.plus_seconds( PRIOR_TO_SEASON_START_PADDING + 300);
    let season_1_end_date = season_1_start_date.plus_seconds(THIRTY_MINUTES);
    let season_1 = get_season(season_id, season_1_start_date.clone(), season_1_end_date.clone());


    season_id = season_id + 1u64;
    let season_2_start_date = season_1_start_date.plus_seconds(FIFTEEN_MINUTES);
    let season_2_end_date = season_1_end_date.plus_seconds(FIFTEEN_MINUTES);
    let season_2 = get_season(season_id, season_2_start_date, season_2_end_date);

    season_id = season_id + 1u64;
    let season_3_start_date = season_2_start_date.minus_seconds(ONE_MINUTE);
    let season_3_end_date = season_2_end_date.minus_seconds(ONE_MINUTE);
    let season_3 = get_season(season_id, season_3_start_date, season_3_end_date);



    let add_season_to_user1_league_res = add_season_to_league(app.borrow_mut(),
                                                         USER1,  season_1, user1_league_addr[0].clone()).unwrap();

    let add_season_to_user2_league_res = add_season_to_league(app.borrow_mut(),
                                                         USER2,  season_2, user2_league_addr[0].clone()).unwrap();

    let add_season_to_user3_league_res = add_season_to_league(app.borrow_mut(),
                                                             USER3,  season_3, user3_league_addr[0].clone()).unwrap();



    let season_2_id = season_id - 1;
    team_request_to_join_league(app.borrow_mut(), USER3, user3_team_addr[0].clone(),  season_2_id).unwrap();


   let request_res =  team_request_to_join_league(app.borrow_mut(), USER3, user3_team_addr[0].clone(), season_id);
    match request_res {
        Ok(_) => {
            assert!(false)
        }
        Err(e) => {

            assert_eq!(goi_manager::ContractError::SeasonScheduleConflict { conflicting_season: vec![season_2_id] }, e.downcast::<ContractError>().unwrap())

        }
    }


}


#[test]
fn team_season_join_success() {

    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1, USER2, USER3], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let block_time =  mock_env().block.time.clone();


    let goi_manager_addr = instantiate_management_contract(&mut app);

    let user1_team_addr = get_team_for_users(vec![Addr::unchecked(USER1)],
                                             goi_manager_addr.clone(), app.borrow_mut());

    let user2_team_addr = get_team_for_users(vec![Addr::unchecked(USER2)],
                                             goi_manager_addr.clone(), app.borrow_mut());




    let user1_league_addr = get_league_for_users(vec![Addr::unchecked(USER1)],
                                                 goi_manager_addr.clone(), app.borrow_mut());





    let mut season_id = 1u64;
    let season_1_start_date = block_time.plus_seconds( PRIOR_TO_SEASON_START_PADDING + 300);
    let season_1_end_date = season_1_start_date.plus_seconds(THIRTY_MINUTES);
    let season_1 = get_season(season_id, season_1_start_date.clone(), season_1_end_date.clone());





    let add_season_to_user1_league_res = add_season_to_league(app.borrow_mut(),
                                                              USER1,  season_1, user1_league_addr[0].clone()).unwrap();



                                                              
    let request_to_join_open_season_res =
                team_request_to_join_league(app.borrow_mut(), USER2, user2_team_addr[0].clone(),  season_id);



    match request_to_join_open_season_res {
        Ok(_) => {
            assert!(true)
        }
        Err(_) => {
           
            assert!(false)

        }
    }


}
