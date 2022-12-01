use std::borrow::{Borrow, BorrowMut};
use std::fmt::Error;
use std::ops::Sub;
use cosmwasm_std::{Coin, Uint128, CosmosMsg, BankMsg, Addr, Order, StdResult};
use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage};
use cw4::Member;
use cw_multi_test::{App, AppResponse, Executor};
use cw_storage_plus::{Bound, PrefixBound};
use goi_manager::ContractError;
use goi_manager::ContractError::Unauthorized;
use goi_manager::state::seasons;
use league::msg::ExecuteMsg::{AddSeasonToLeague};
use managed::ManagedServiceError;
use managed::messages::ManagedExecuteMsg;
use saleable::error::SaleableItemError;
use saleable::messages::receive::ExecuteMsg::{Buy, Update};
use shared::data::ModelItem;
use shared::goi_manager::ExecuteMsg::{Withdraw, self, UpdateSeasonStatus};
use shared::goi_manager::GoiManagerQueryMsg;
use shared::goi_manager::GoiManagerQueryMsg::{GetActiveSeasonsForLeague, GetAllSeasonsForLeague, GetLeagueTeams, GetMessagesToItem};
use shared::manage::receive::ManagementInfoResponse;
use shared::messaging::{JoinSeasonRequestInfo, Message, MessageTypes};
use shared::query_response_info::InfoManagedResponse;
use shared::rewards::RewardTypes::League;
use shared::season::{Season, SeasonAccessTypes, SeasonStatus, SeasonModelData};
use shared::team::TeamInfo;
use shared::utils::general::AssetTypes;
use shared::utils::{AssetSaleItem, ManagedItemResponse, MangedItem, PRIOR_TO_SEASON_START_PADDING, SeasonId, THIRTY_MINUTES, FIFTEEN_MINUTES, MAX_TEAMS_ALLOWED};
use team::msg::ExecuteMsg::{ManagedServiceMessage, JoinLeague};
use team::TeamError;

use crate::shared_utils::{TOKEN, mock_app_by_user, USER1, OWNER, instantiate_management_contract, USER2, instantiate_team_with_managed_contract, member, mock_app, instantiate_management_contract_with_user, get_team_for_sale, buy_team, instantiate_league_with_managed_contract, add_owner_teams_to_league, instantiate_team_with_managed_contract_with_sender_admin, USER3};



#[test]
fn with_draw_funds() {
    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1], &[init_balance.clone()]);

    let goi_manager_addr1 = instantiate_management_contract(&mut app);


    let send_amount_to_goi_manager = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000u128) };
    let with_draw_amount = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(2500000u128) };
    let send_to_goi_manager_msg =
            CosmosMsg::Bank(BankMsg::Send {
                to_address: goi_manager_addr1.clone().to_string(),
                amount: vec![send_amount_to_goi_manager.clone()],
            });
    app.execute( Addr::unchecked(USER1), send_to_goi_manager_msg).unwrap();


    app.wrap().query_all_balances(goi_manager_addr1.clone()).unwrap();

    let withdraw_msg = Withdraw { recipient: Addr::unchecked(USER2) , amount: vec![with_draw_amount.clone()] };

    app.execute_contract(Addr::unchecked(OWNER),
                            goi_manager_addr1.clone(), &withdraw_msg.clone(),
                                &[]).unwrap();

    let goi_manager_balance =
                    app.wrap().query_all_balances(goi_manager_addr1.clone()).unwrap();                           

    let user_2_balance =
        app.wrap().query_all_balances(USER2.clone()).unwrap();

    assert_eq!(goi_manager_balance[0].amount, send_amount_to_goi_manager.clone().amount - with_draw_amount.clone().amount);
    assert_eq!(goi_manager_balance[0].denom, with_draw_amount.clone().denom);           

    assert_eq!(user_2_balance[0].amount, with_draw_amount.clone().amount);
    assert_eq!(user_2_balance[0].denom, with_draw_amount.clone().denom);

    //Let's try withdraw with unauthorized sender
    let withdraw_err = app.execute_contract(Addr::unchecked(USER2.clone()),
                            goi_manager_addr1.clone(), &withdraw_msg,
                                &[]).unwrap_err();

   
    assert_eq!( goi_manager::ContractError::Unauthorized { sender: Addr::unchecked(USER2) },
        withdraw_err.downcast().unwrap());                            
                     
}

#[test]
fn get_active_seasons() {
    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let goi_manager_addr1 = instantiate_management_contract(&mut app);
    let league1 = instantiate_league_with_managed_contract(&mut app, OWNER.clone().to_string(), 
                vec![Member { addr: OWNER.clone().to_string(), weight: 100 }], Some(goi_manager_addr1.clone()));
    let block_time =  mock_env().block.time.clone();
    let a_season =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our team!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(block_time.plus_seconds(300)) },
            end_date: ModelItem { update: true, data: Some(block_time.plus_seconds((300 * 2))) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED) },
          
        };

    let add_season_to_league_msg =
        AddSeasonToLeague { season_name: "Big House".to_string(), season_model: a_season };

    let add_season_to_league_res =
            app.execute_contract(Addr::unchecked(OWNER),
                    league1.clone(),
                    &add_season_to_league_msg, &[]);
    match add_season_to_league_res {
        Ok(_) =>
            {
                let managed_items_res:Option<Vec<Season>> = app
                    .wrap()
                    .query_wasm_smart(
                        &goi_manager_addr1,
                        &  GetAllSeasonsForLeague { league_address: league1 } )
                    .unwrap();
                match managed_items_res {
                    None => { assert!(false) }
                    Some(r) => {
                            assert_eq!(r.len(), 1)
                    }
                }

            },
        Err(_) => assert!(false),
    }
}



#[test]
fn add_teams_to_league() {
    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(15000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1, USER2, USER3], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let goi_manager_addr1 = instantiate_management_contract(&mut app);
    let league1 = instantiate_league_with_managed_contract(&mut app, OWNER.clone().to_string(),
                                                           vec![Member { addr: OWNER.clone().to_string(), weight: 100 }], Some(goi_manager_addr1.clone()));



    let league2 = instantiate_league_with_managed_contract(&mut app, USER1.clone().to_string(),
                                                           vec![Member { addr: USER1.clone().to_string(), weight: 100 }], Some(goi_manager_addr1.clone()));    
                                                           
                                                           
    let block_time =  mock_env().block.time.clone();



    let season_1_start_date = block_time.plus_seconds( PRIOR_TO_SEASON_START_PADDING + 300);
    let season_1_end_date = season_1_start_date.plus_seconds(THIRTY_MINUTES);
    let season_1 =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our season 1!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(season_1_start_date) },
            end_date: ModelItem { update: true, data: Some(season_1_end_date) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED)},
        };


    let season_2_start_date = season_1_end_date.plus_seconds(FIFTEEN_MINUTES);
    let season_2_end_date = season_2_start_date.plus_seconds(THIRTY_MINUTES);
    let season_2 =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our season 2!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(season_2_start_date) },
            end_date: ModelItem { update: true, data: Some( season_2_end_date ) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },

            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED) },

        };



    let season_3_start_date = season_2_end_date.plus_seconds(FIFTEEN_MINUTES);
    let season_3_end_date = season_3_start_date.plus_seconds(THIRTY_MINUTES);
    let season_3 =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our season 3!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(season_3_start_date) },
            end_date: ModelItem { update: true, data: Some( season_3_end_date ) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED) },
        };

    let add_season_to_league_msg =
        AddSeasonToLeague { season_name: "Big House".to_string(), season_model: season_1 };

    let add_season_to_league_res =
        app.execute_contract(Addr::unchecked(OWNER),
                             league1.clone(),
                             &add_season_to_league_msg, &[]);


    let add_season2_to_league_msg =
                             AddSeasonToLeague { season_name: "Big House 2".to_string(), season_model: season_2 };
                     
    let add_season2_to_league_res =
                             app.execute_contract(Addr::unchecked(USER1),
                                                  league2.clone(),
                                                  &add_season2_to_league_msg, &[]);



    let add_season3_to_league_msg =
        AddSeasonToLeague { season_name: "Big House 3".to_string(), season_model: season_3 };

    let add_season3_to_league_res =
        app.execute_contract(Addr::unchecked(OWNER),
                             league1.clone(),
                             &add_season3_to_league_msg, &[]);


    match (add_season_to_league_res, add_season2_to_league_res) {
        (Ok(_), Ok(_)) =>
            {
                let managed_season_items_res:Option<Vec<Season>> = app
                    .wrap()
                    .query_wasm_smart(
                        &goi_manager_addr1,
                        &  GetAllSeasonsForLeague { league_address: league1.clone() } )
                    .unwrap();

/*
                let managed_season_items_res_2:Option<Vec<Season>> = app
                    .wrap()
                    .query_wasm_smart(
                        &goi_manager_addr1,
                        &  GetAllSeasonsForLeague { league_address: league2.clone() } )
                    .unwrap();
                */
                match managed_season_items_res {
                    None => { assert!(false) }
                    Some(r) => {
                        assert_eq!(r.len(), 2);

                        let team_addr1 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_addr2 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_addr3 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));


                        let team_user1_addr1 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_user1_addr2 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_user2_addr3 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));


                        let attempt_to_add_non_owned_teams_to_league_err =
                            add_owner_teams_to_league(&mut app, vec![team_addr1.clone(), team_addr2.clone(), team_addr3.clone()],
                                                      league1.clone(), Addr::unchecked(USER1));

                        match attempt_to_add_non_owned_teams_to_league_err {
                            Ok(_) => assert!(false),
                            Err(_) => assert!(true)
                        }


                        add_owner_teams_to_league(&mut app, vec![team_addr1, team_addr2, team_addr3],
                                                  league1.clone(), Addr::unchecked(OWNER)).unwrap();


                        let league_teams_res: Option<Vec<TeamInfo>> = app
                            .wrap()
                            .query_wasm_smart(
                                &goi_manager_addr1.clone(),
                                &GetLeagueTeams { league_addr: league1.clone() })
                            .unwrap();

                        match league_teams_res {
                            None => {
                                assert!(false)
                            }
                            Some(t) => {
                                assert_eq!(t.len(), 3)
                            }
                        }


                        //sell team for later usage in "invite" unit tests below -- BEGIN
                        let team_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500000u128) };
                        let for_sale_msg =
                            ManagedServiceMessage {
                                message: ManagedExecuteMsg::Saleable {
                                    saleable_msg: Update {
                                        for_sale_status: true,
                                        price: Some(team_sell_price.clone())
                                    }
                                }
                            };


                        app.execute_contract(Addr::unchecked(OWNER),
                                             team_user1_addr1.clone(), &for_sale_msg, &[]).unwrap();


                        app.execute_contract(Addr::unchecked(OWNER),
                                             team_user1_addr2.clone(), &for_sale_msg, &[]).unwrap();

                        app.execute_contract(Addr::unchecked(OWNER),
                                             team_user2_addr3.clone(), &for_sale_msg, &[]).unwrap();




                        let buy_team_msg = ManagedServiceMessage { message: ManagedExecuteMsg::Saleable { saleable_msg: Buy {} } };


                        let mut res_sell_ok =
                            app.execute_contract(Addr::unchecked(USER1),
                                                 team_user1_addr1.clone(), &buy_team_msg,
                                                 &[team_sell_price.clone()]).unwrap();

                        res_sell_ok =
                            app.execute_contract(Addr::unchecked(USER1),
                                                 team_user1_addr2.clone(), &buy_team_msg,
                                                 &[team_sell_price.clone()]).unwrap();


                        res_sell_ok =
                            app.execute_contract(Addr::unchecked(USER2),
                                                 team_user2_addr3.clone(), &buy_team_msg,
                                                 &[team_sell_price.clone()]).unwrap();

                        //sell team for later usage in "invite" unit tests below -- END




                        //sell league for later usage in "invite" unit tests below -- BEGIN
                        let league_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500000u128) };
                        let league_for_sale_msg =
                            ManagedServiceMessage {
                                message: ManagedExecuteMsg::Saleable {
                                    saleable_msg: Update {
                                        for_sale_status: true,
                                        price: Some(league_sell_price.clone())
                                    }
                                }
                            };

                        app.execute_contract(Addr::unchecked(OWNER),
                                                 league1.clone(), &league_for_sale_msg, &[]).unwrap();

                        let buy_league_msg = ManagedServiceMessage { message: ManagedExecuteMsg::Saleable { saleable_msg: Buy {} } };

                        app.execute_contract(Addr::unchecked(USER1),
                                                 league1.clone(), &buy_league_msg,
                                                 &[league_sell_price.clone()]).unwrap();

                        //sell league for later usage in "invite" unit tests below -- END

                        let get_user_leagues_res: Option<Vec<ManagedItemResponse>> = app
                            .wrap()
                            .query_wasm_smart(
                                &goi_manager_addr1.clone(),
                                &GoiManagerQueryMsg::GetOwnerAssets {
                                    owner_address: Addr::unchecked(USER1)
                                })
                            .unwrap();

                        match get_user_leagues_res {
                            None => {}
                            Some(items) => {
                                assert_eq!( items.len(), 3)
                            }
                        }

                        let team_request_to_join_league = JoinLeague {  season_id: 1 };
                        let team_request_to_join_league_res =
                            app.execute_contract(Addr::unchecked(USER1),
                                                 team_user1_addr1.clone(),
                                                 &team_request_to_join_league, &[]);



                        let team_request_to_join_league1_season_3 = JoinLeague {  season_id: 2 };
                        let team_request_to_join_league1_season_3_res =
                            app.execute_contract(Addr::unchecked(USER1),
                                                 team_user1_addr1.clone(),
                                                 &team_request_to_join_league1_season_3, &[]);



                        let team_request_to_join_league1_season_4 = JoinLeague {  season_id: 2 };
                        let team_request_to_join_league1_season_4_res =
                            app.execute_contract(Addr::unchecked(USER2),
                                                 team_user2_addr3.clone(),
                                                 &team_request_to_join_league1_season_4, &[]);

                        let league1_invites_request_messages_received_res: Option<Vec<Message<JoinSeasonRequestInfo>>> = app
                            .wrap()
                            .query_wasm_smart(
                                &goi_manager_addr1.clone(),
                                &GetMessagesToItem {
                                    item_addr: league1.clone(),
                                    asset_type: AssetTypes::League
                                })
                            .unwrap();
                        match league1_invites_request_messages_received_res {
                            Some(invites) => {
                                assert_eq!(invites.len() , 1)
                            },
                            None => assert!(false),
                        }


                        let get_active_seasons_for_league_res: Option<Vec<Season>> = app
                            .wrap()
                            .query_wasm_smart(
                                &goi_manager_addr1.clone(),
                                &GetAllSeasonsForLeague {
                                    league_address: league1.clone(),
                                })
                            .unwrap();
                        match get_active_seasons_for_league_res {
                            Some(items) => {
                                assert_eq!(items.len() , 2)
                            },
                            None => assert!(false),
                        }

                    }
                }

            },
        _ => assert!(false),
    }
}


/*




#[test]
fn add_teams_to_league() {
    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1], &[init_balance.clone()]);
    let mut store = MockStorage::new();
    let goi_manager_addr1 = instantiate_management_contract(&mut app);
    let league1 = instantiate_league_with_managed_contract(&mut app, OWNER.clone().to_string(),
                                                           vec![Member { addr: OWNER.clone().to_string(), weight: 100 }], Some(goi_manager_addr1.clone()));



    let league2 = instantiate_league_with_managed_contract(&mut app, OWNER.clone().to_string(),
                                                           vec![Member { addr: USER1.clone().to_string(), weight: 100 }], Some(goi_manager_addr1.clone()));    
                                                           
                                                           
    let block_time =  mock_env().block.time.clone();


    let a_season =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our team!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(block_time.plus_seconds(300)) },
            end_date: ModelItem { update: true, data: Some(block_time.plus_seconds((300 * 2))) },
            season_type: ModelItem { update: true, data: Some(SeasonTypes::NotSet) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            content_status: ModelItem { update: true, data: Some( Visibility::Published) },
            max_teams_allowed: ModelItem { update: true, data: Some(32) },
            number_of_episodes: ModelItem { update: true, data: Some(20) }
        };


    let a_season2 =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("This is our team!!".to_string()), },
            start_date: ModelItem { update: true, data: Some(block_time.plus_seconds(300)) },
            end_date: ModelItem { update: true, data: Some(block_time.plus_seconds((300 * 2))) },
            season_type: ModelItem { update: true, data: Some(SeasonTypes::NotSet) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            content_status: ModelItem { update: true, data: Some( Visibility::Published) },
            max_teams_allowed: ModelItem { update: true, data: Some(32) },
            number_of_episodes: ModelItem { update: true, data: Some(20) }
        };

    let add_season_to_league_msg =
        AddSeasonToLeague { season_name: "Big House".to_string(), season_model: a_season };

    let add_season_to_league_res =
        app.execute_contract(Addr::unchecked(OWNER),
                             league1.clone(),
                             &add_season_to_league_msg, &[]);


    let add_season2_to_league_msg =
                             AddSeasonToLeague { season_name: "Big House 2".to_string(), season_model: a_season2 };
                     
    let add_season2_to_league_res =
                             app.execute_contract(Addr::unchecked(USER1),
                                                  league1.clone(),
                                                  &add_season2_to_league_msg, &[]);


    match (add_season_to_league_res, add_season2_to_league_res) {
        (Ok(_), Ok(_)) =>
            {
                let managed_season_items_res:Option<Vec<Season>> = app
                    .wrap()
                    .query_wasm_smart(
                        &goi_manager_addr1,
                        &  GetAllSeasonsForLeague { league_address: league1.clone() } )
                    .unwrap();
                match managed_season_items_res {
                    None => { assert!(false) }
                    Some(r) => {
                        assert_eq!(r.len(), 1);

                        let team_addr1 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_addr2 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let team_addr3 =
                            instantiate_team_with_managed_contract
                                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr1.clone()));

                        let attempt_to_add_non_owner_teams_to_league_err =
                            add_owner_teams_to_league(&mut app, vec![team_addr1.clone(), team_addr2.clone(),team_addr3.clone()],
                                                      league1.clone(), Addr::unchecked(USER1));

                        match attempt_to_add_non_owner_teams_to_league_err {
                            Ok(_) => assert!(false),
                            Err(_) => assert!(true)
                        }
                      

                        add_owner_teams_to_league(&mut app, vec![team_addr1, team_addr2,team_addr3],
                                                    league1.clone(), Addr::unchecked(OWNER)).unwrap();


                        let league_teams_res :Option<Vec<TeamInfo>> = app
                            .wrap()
                            .query_wasm_smart(
                                &goi_manager_addr1,
                                &  GetLeagueTeams { league_addr: league1 } )
                            .unwrap();

                        match league_teams_res {
                            None => {
                                assert!(false)
                            }
                            Some(t) => {
                                assert_eq!(t.len(), 3)
                            }
                        }

                        let invite_team_msg = ExecuteMsg::InviteTeamsToLeague { teams: (), season_id: (), sending_user: () };
                    }
                }

            },
        _ => assert!(false),
    }
}


*/



#[test]
fn get_teams_for_sale() {

    let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
    let mut app: App = mock_app_by_user(vec![OWNER, USER1], &[init_balance.clone()]);
    let goi_manager_addr1 =
        instantiate_management_contract_with_user
            (&mut app, OWNER.to_string()).unwrap();

    let team_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500000u128) };

    let team1 = get_team_for_sale(app.borrow_mut(), goi_manager_addr1.clone(), team_sell_price.clone()).unwrap();
    let team2 = get_team_for_sale(app.borrow_mut(), goi_manager_addr1.clone(), team_sell_price.clone()).unwrap();
    let team3 = get_team_for_sale(app.borrow_mut(), goi_manager_addr1.clone(), team_sell_price.clone()).unwrap();
    let team4 = get_team_for_sale(app.borrow_mut(), goi_manager_addr1.clone(), team_sell_price.clone()).unwrap();
    let team5 = get_team_for_sale(app.borrow_mut(), goi_manager_addr1.clone(), team_sell_price.clone()).unwrap();

    buy_team(app.borrow_mut(), Addr::unchecked(USER1), team3, team_sell_price);


    let teams_for_sale:Option<Vec<ManagedItemResponse>> = app
        .wrap()
        .query_wasm_smart(
            &goi_manager_addr1,
            &GoiManagerQueryMsg::GetAssetsForSale
            { contract_type: AssetTypes::Team })
        .unwrap();

    match teams_for_sale {
        None => {
            assert!(false)
        }
        Some(items) => {
            assert_eq!(items.len(), 4);
        }
    }

}