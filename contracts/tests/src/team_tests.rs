
/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Coin, coins, Empty, from_binary, StdResult, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use anyhow::Error;
    use application::ContractError;
    use application::msg::ExecuteMsg::{AddNewTask, UpdateTaskCodeId};
    use shared::application::ApplicationQueryMsg;
    use shared::task::{TaskCreateModel, TaskInfoResponse};
    use shared::utils::BlockTime;
    use task::state::Task;
    use crate::shared_utils::{CHAIN_ID, COIN_DENOM, INIT_ADMIN, instantiate_management_contract, instantiate_management_contract_with_user, mock_app, OWNER};
    use crate::shared_utils::apps::{contract_task, instantiate_application_contract, instantiate_task_contract_by_owning_user};


    #[test]
    fn exec_msg () {
        let mut app = mock_app(&[]);
        let goi_manager_addr1 = instantiate_management_contract(&mut app);
        let application_addr = instantiate_application_contract(&mut app, "RUNNERS".to_string(), Some(goi_manager_addr1.clone()));


        let task_code_id =  app.store_code(contract_task());

        //AddNewTask{ task: TaskCreateModel },
        let update_task_code_id_msg =  UpdateTaskCodeId{ task_code_id: Some(task_code_id)};
        let update_task_code_id_res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 application_addr.clone(), &update_task_code_id_msg, &[]);

        let current_time = Timestamp::default();
        let bound_amount = Coin{ denom: COIN_DENOM.to_string(), amount: Uint128::from(5000u128) };
        let add_new_task_msg = AddNewTask{ task: TaskCreateModel {
            name: "My new task".to_string(),
            description: Some( "This is my description.".to_string()),
            start_date: BlockTime {
                height: 0,
                time: current_time,
                chain_id: CHAIN_ID.to_string()
            },
            end_date: Some(BlockTime{
                height: 1,
                time: current_time.plus_seconds(60),
                chain_id: CHAIN_ID.to_string()
            }),
            reward_threshold: 0,
            bond_amount: vec![bound_amount.clone()],
            exec_msg: Some("{}".to_string()),
            target_executable_contact: goi_manager_addr1.clone(),
            task_id: "".to_string()
        } };


        let add_new_task_res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 application_addr.clone(), &add_new_task_msg, &[]);

        match add_new_task_res {
            Ok(_) => {
                let get_task_msg = ApplicationQueryMsg::GetTask{ task_id: u8::from(1), xnode_address: None };
                let task: Option<TaskInfoResponse> = app
                    .wrap()
                    .query_wasm_smart(
                        &application_addr.clone(),
                        &get_task_msg ).unwrap();
                assert_eq!(task.clone().unwrap().task_info.task_id,  u8::from(1));
                assert_eq!(task.unwrap().task_info.bound_amount.get(0).unwrap(),  &bound_amount);
            }
            Err(_) => {
                assert!(false)
            }
        }
        //let team_sell_price = Coin { denom: COIN_DENOM.to_string(), amount: Uint128::from(500000u128) };
        //let wrong_sell_price = Coin { denom: COIN_DENOM.to_string(), amount: Uint128::from(500u128) };

    }


}
 */



#[cfg(test)]
mod tests {
    use std::ops::{Mul, Sub};

    use anyhow::Error;
    use cosmwasm_std::{Coin, Empty, from_slice, OwnedDeps, Querier, Storage, SubMsg, Uint128, Addr, DepsMut, Api, BankMsg, coins, CosmosMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw4::{Member, member_key, MemberChangedHookMsg, MemberDiff, TOTAL_KEY};
    use cw4_group::contract::update_members;
    use cw4_group::state::MEMBERS;
    use cw_controllers::{AdminError, HookError};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use cw_multi_test::AppBuilder;


    use group_admin::GroupAdminError;
    use group_admin::messages::receive::ExecuteMsg::{AddHook, RemoveHook, UpdateMembers};
    use group_admin::service::list_members;
    use managed::ManagedServiceError;
    use managed::messages::ManagedExecuteMsg;
    use shared::player::{InstantiateMsg, PlayerInfo};
    use saleable::error::SaleableItemError;
    use saleable::messages::receive::ExecuteMsg::{Buy, Update};
    use shared::goi_manager::GoiManagerQueryMsg;
    use shared::goi_manager::ManagementQryMsg::GetManagementInfo;
    use shared::manage::receive::ManagementInfoResponse;
    use shared::player_attributes::{AttrItem, PlayerAttributes, Positions};
    use shared::query_response_info::InfoManagedResponse;
    use shared::utils::general::AssetTypes;
    use shared::utils::{AssetSaleItem, ManagedItemResponse};
    use team::TeamError;
    use team::msg::InstantiateTeamMsg;

    use team::msg::ExecuteMsg::{AddPlayersToTeam, ManagedServiceMessage, RemovePlayersFromTeam};
    use crate::shared_utils::{all_players, all_players_with_duplicate_name, assert_users, build_player_contracts, do_instantiate_team, get_player_instantiate_msg, INIT_ADMIN, instantiate_management_contract, instantiate_management_contract_with_user, instantiate_player, instantiate_team_with_managed_contract, member, mock_app, mock_app_by_user, OWNER, TOKEN, USER1, USER2, USER3};




    #[test]
    fn add_player_to_filled_position() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
        //let add_managed_contract_msg = AddManagedContract { asset_name: Some("Team #1".to_string()), asset_owner: Addr::unchecked( OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };
        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
        //                                                        goi_manager_addr.clone(), &add_managed_contract_msg, &[]);
        let player_msgs =
            build_player_contracts(&mut app, all_players(goi_manager_addr.clone()), OWNER).unwrap();
        let players_msg = AddPlayersToTeam { players: player_msgs };

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &players_msg, &[]);

        let init_mg = get_player_instantiate_msg("WR19".to_string(), "Hash".to_string(),
                                                 Positions::WR1, goi_manager_addr);
        let single_player_msg_res =
            build_player_contracts(&mut app, vec![init_mg], OWNER);
        match single_player_msg_res {
            Ok(single_player_msg) => {
                let player_msg = AddPlayersToTeam { players: single_player_msg };

                let position_already_assigned_err =
                    app.execute_contract(Addr::unchecked(OWNER),
                                         team_addr, &players_msg, &[]).unwrap_err();

                assert_eq!(TeamError::PositionAlreadyAssigned {},
                           position_already_assigned_err.downcast().unwrap());
            }
            Err(_) => {
                assert!(false)
            }
        }

    }


    #[test]
    fn get_managed_contract() {
        let mut app = mock_app(&[]);
        let goi_manager_addr = instantiate_management_contract(&mut app);
        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)],
             Some(goi_manager_addr.clone()));
        let update_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::UpdateManager
        { manager_address: goi_manager_addr.clone()}};
        let update_managed_status_msg =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &update_msg, &[]);

        let managed_items_res: Option<Vec<ManagedItemResponse>> = app
            .wrap()
            .query_wasm_smart(
                &goi_manager_addr,
                & GoiManagerQueryMsg::GetOwnerAssets { owner_address: Addr::unchecked(OWNER) }, )
            .unwrap();
        match managed_items_res {
            None => {
                assert!(false)
            }
            Some(r) => {
                assert_eq!(r.len(), 1);
                assert_eq!(r[0].for_sale, false);
            }
        }


    }




    #[test]
    fn set_managed_contract() {
        let mut app = mock_app(&[]);
        let goi_manager_addr = instantiate_management_contract(&mut app);
        let wrong_goi_manager_addr = instantiate_management_contract(&mut app);
        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)],
             Some(goi_manager_addr.clone()));
        let update_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::UpdateManager
            { manager_address: goi_manager_addr.clone()}};
        let update_managed_status_msg =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &update_msg, &[]);

        let contract: InfoManagedResponse<team::state::State> = app
            .wrap()
            .query_wasm_smart(
                &team_addr,
                &team::msg::QueryMsg::GetInfo {}, )
            .unwrap();

        assert_eq!(contract.managed_info.managing_contract == Some(goi_manager_addr), true);
    }


    #[test]
    fn test_instantiate_goi_manager() {
        let mut app = mock_app(&[]);
        let goi_manager_addr1 = instantiate_management_contract(&mut app);
        let goi_manager_addr2_res =
            instantiate_management_contract_with_user
                (&mut app, INIT_ADMIN.to_string()).unwrap_err();
        assert_eq!(goi_manager::ContractError::Unauthorized { sender: Addr::unchecked(INIT_ADMIN) },
                   goi_manager_addr2_res.downcast().unwrap());
    }


    #[test]
    fn sell_team() {
        let init_balance = Coin { denom: TOKEN.parse().unwrap(), amount: Uint128::from(5000000000000u128) };
        let mut app: App = mock_app_by_user(vec![OWNER, USER1], &[init_balance.clone()]);

        let goi_manager_addr1 = instantiate_management_contract(&mut app);

        let team_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500000u128) };
        let wrong_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500u128) };

        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)],
             Some(goi_manager_addr1.clone()));
        let for_sale_msg =
            ManagedServiceMessage{ message: ManagedExecuteMsg::Saleable {
                saleable_msg: Update {
                    for_sale_status: true,
                    price: Some(team_sell_price.clone())
                }
            }};

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &for_sale_msg, &[]);






        let buy_team_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable { saleable_msg: Buy {} }};


        let res_err =
            app.execute_contract(Addr::unchecked(USER1),
                                 team_addr.clone(), &buy_team_msg,
                                 &[wrong_sell_price]).unwrap_err();

        assert_eq!( TeamError::ManagableServiceError{
            0: ManagedServiceError::SaleItemErrors{
                0: SaleableItemError::InsufficientFundsSend {}
            }
        }, res_err.downcast().unwrap());

        let res_sell_ok =
            app.execute_contract(Addr::unchecked(USER1),
                                 team_addr.clone(), &buy_team_msg,
                                 &[team_sell_price.clone()]).unwrap();

        

        let team_contract: InfoManagedResponse<team::state::State> = app
            .wrap()
            .query_wasm_smart(
                &team_addr,
                &team::msg::QueryMsg::GetInfo {}, )
            .unwrap();

        let goi_manager_contract_info: ManagementInfoResponse = app
            .wrap()
            .query_wasm_smart(
                &goi_manager_addr1,
                &GoiManagerQueryMsg::ManagementQryMessages
                { management_qry_msg: GetManagementInfo {} })
            .unwrap();

        match team_contract.admin {
            None => assert!(false),
            Some(admin_addr) => assert_eq!(admin_addr, Addr::unchecked(USER1))
        }


        let goi_manager_contract_balance =
            app.wrap().query_all_balances(goi_manager_addr1.clone()).unwrap();

        let goi_manager_fee_payment =
            goi_manager_contract_info.fees.unwrap()[0].fees.percent.mul(team_sell_price.amount);
        assert_eq!(goi_manager_contract_balance[0].denom, team_sell_price.clone().denom);
        assert_eq!(goi_manager_contract_balance[0].amount.clone(), goi_manager_fee_payment);

        let seller_balance = app.wrap().query_all_balances(OWNER).unwrap();
        let buyer_balance = app.wrap().query_all_balances(USER1).unwrap();

        assert_eq!(init_balance.amount.sub(buyer_balance[0].amount),
                   team_sell_price.amount);
        assert_eq!(seller_balance[0].amount.sub(init_balance.amount),
                   team_sell_price.amount.sub(goi_manager_fee_payment));

        assert_eq!(team_contract.owners.len() == 1, true);
        assert_eq!(team_contract.owners[0].addr, USER1.to_string());

        let managed_items_res: Option<Vec<ManagedItemResponse>> = app
                .wrap()
                .query_wasm_smart(
                    &goi_manager_addr1,
                    & GoiManagerQueryMsg::GetOwnerAssets { owner_address: Addr::unchecked(USER1) }, )
                .unwrap();
        match managed_items_res {
            None => {
                assert!(false)
            }
            Some(r) => {
                assert_eq!(r.len(), 1);
                assert_eq!(r[0].for_sale, false);
            }
        }

    }


    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();

        do_instantiate_team(deps.as_mut(), false, None, None);

        // it worked, let's query the state
        let res = team::state::ADMIN.query_admin(deps.as_ref()).unwrap();
        assert_eq!(Some(INIT_ADMIN.into()), res.admin);

        let res = team::contract::query_total_weight(deps.as_ref()).unwrap();
        assert_eq!(17, res.weight);
    }

    #[test]
    fn test_instantiate_team() {
        let mut app = mock_app(&[]);
        let goi_manager_addr1 = instantiate_management_contract(&mut app);
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 1)], None);
        let player_addr_res =
            instantiate_player(&mut app, get_player_instantiate_msg("QB".to_string(), "Hash".to_string(),
                                                                    Positions::QB, goi_manager_addr1), OWNER); // app.store_code(contract_player());
        match player_addr_res {
            Ok(player_addr) => {
                assert_ne!(player_addr, team_addr);
            }
            Err(_) => {
                assert!(false)
            }
        }

    }

    #[test]
    fn try_member_queries() {
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        let member1 =
            team::contract::query_member(deps.as_ref(), USER1.into(), None).unwrap();
        assert_eq!(member1.weight, Some(11));

        let member2 =
            team::contract::query_member(deps.as_ref(), USER2.into(), None).unwrap();
        assert_eq!(member2.weight, Some(6));

        let member3 =
            team::contract::query_member(deps.as_ref(), USER3.into(), None).unwrap();
        assert_eq!(member3.weight, None);

        let members =
            group_admin::service::list_members(deps.as_ref(), None, None, MEMBERS).unwrap();
        assert_eq!(members.members.len(), 2);
        // TODO: assert the set is proper
    }


    #[test]
    fn set_team_for_sale_with_no_price() {
        let mut app = mock_app(&[]);
        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)], None);
        let for_sale_with_no_price_set_msg =
            ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable
            { saleable_msg: Update { for_sale_status: true, price: None } }};

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &for_sale_with_no_price_set_msg,
                                 &[]).unwrap_err();


        assert_eq!(TeamError::ManagableServiceError{
                        0: ManagedServiceError::SaleItemErrors{
                            0: SaleableItemError::PriceNotSet {}
                        }
                    },
                   res.downcast().unwrap());

        let contract: InfoManagedResponse<team::state::State> = app
            .wrap()
            .query_wasm_smart(
                &team_addr,
                &team::msg::QueryMsg::GetInfo {}, )
            .unwrap();

        assert_eq!(contract.sale_info.price.is_none() && !contract.sale_info.for_sale, true);
    }


    #[test]
    fn purchase_team() {
        let mut app = mock_app(&[]);
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], None);
        let for_sale_with_no_price_set_msg =
            ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable { saleable_msg: Update { for_sale_status: true, price: None } }};


        let res =
            app.execute_contract
            (Addr::unchecked(OWNER),
             team_addr.clone(),
             &for_sale_with_no_price_set_msg, &[]).unwrap_err();


        assert_eq!( TeamError::ManagableServiceError{
                        0: ManagedServiceError::SaleItemErrors{
                            0: SaleableItemError::PriceNotSet {}
                        }
                    },
                   res.downcast().unwrap());

        let contract: InfoManagedResponse<team::state::State> = app
            .wrap()
            .query_wasm_smart(
                &team_addr,
                &team::msg::QueryMsg::GetInfo {}, )
            .unwrap();

        assert_eq!(contract.sale_info.price.is_none() && !contract.sale_info.for_sale, true);
    }

    #[test]
    fn set_team_for_sale_with_price_of_zero() {
        let mut app = mock_app(&[]);
        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)], None);
        let a_price = Some(Coin {
            denom: "token".to_string(),
            amount: Uint128::from(0u128)
        });
        let for_sale_with_price_of_zero_msg =
            ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable
            { saleable_msg: Update { for_sale_status: true, price: a_price.clone() } }};


        let for_sale_with_price_of_zero_res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(),
                                 &for_sale_with_price_of_zero_msg, &[]);
        match for_sale_with_price_of_zero_res {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn set_team_not_for_sale_with_price() {
        let mut app = mock_app(&[]);
        let team_addr = instantiate_team_with_managed_contract
            (&mut app, vec![member(OWNER, 100)], None);
        let a_price = Some(Coin {
            denom: "token".to_string(),
            amount: Uint128::from(5000u128)
        });
        let not_for_sale_with_price_set_msg =
            ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable
            { saleable_msg: Update { for_sale_status: false, price: a_price.clone() } }};


        let not_for_sale_with_price_set_res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(),
                                 &not_for_sale_with_price_set_msg, &[]);
        match not_for_sale_with_price_set_res {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }


        let contract: InfoManagedResponse<team::state::State> = app
            .wrap()
            .query_wasm_smart(
                &team_addr,
                &team::msg::QueryMsg::GetInfo {}, )
            .unwrap();

        assert_eq!(contract.sale_info.price == a_price && !contract.sale_info.for_sale, true);
    }



    #[test]
    fn illegal_position_assignment() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], None);
        let player_msgs =
            build_player_contracts(&mut app, vec![get_player_instantiate_msg("WR1".to_string(), "Hash".to_string(),
                                                                             Positions::WR1, goi_manager_addr)], OWNER).unwrap();
        //attempt to change player's assigned position from wr1 to rb
        let illegal_pos_assignment_msg =
            vec![PlayerInfo {
                first_name: player_msgs[0].first_name.clone(),
                last_name: player_msgs[0].last_name.clone(),
                address: player_msgs[0].address.clone(),
                position: Positions::RB,
                assigned_team_address: None
            }];
        let players_msg = AddPlayersToTeam { players: illegal_pos_assignment_msg };

        let illegal_pos_assignment_err =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &players_msg, &[]).unwrap_err();
        assert_eq!(TeamError::PlayerDeclaredPosAndAssignPosMisMatch {
            player_address: Addr::unchecked("contract2".to_string()),
            position: Positions::RB,
            contract_first_name: "WR1".to_string(),
            contract_last_name: "Hash".to_string(),
            contract_position: Positions::WR1
        }, illegal_pos_assignment_err.downcast().unwrap());
    }


    #[test]
    fn nonexistent_contract_address() {
        let mut app = mock_app(&[]);
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)],
                 None);
        //attempt to assign player with nonexistent contract address
        let unknown_contract_address_msg = vec![PlayerInfo
        { first_name: "TEST_first_name".to_string(), last_name: "TEST_last_name".to_string(), address: Addr::unchecked("[TRASH_ADDRESS]".to_string()), position: Positions::WR1, assigned_team_address: None }];
        let players_msg = AddPlayersToTeam { players: unknown_contract_address_msg };

        let contract_not_found_err =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &players_msg, &[]).unwrap_err();

        let err =
            match contract_not_found_err {
                Error { .. } => TeamError::ContractNotFound {},
            };
        assert_eq!(err, TeamError::ContractNotFound {});
    }


    #[test]
    fn general_players_team_test() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
        //let add_managed_contract_msg = AddManagedContract { asset_name: Some("Team #2".to_string()), asset_owner: Addr::unchecked(OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };
        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
        //                                                        goi_manager_addr.clone(), &add_managed_contract_msg, &[]);
        let player_msgs =
            build_player_contracts(&mut app, all_players(goi_manager_addr), OWNER).unwrap();
        let add_players_msg = AddPlayersToTeam { players: player_msgs.clone() };

        let res1 =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &add_players_msg, &[]);

        let remove_players_msg = RemovePlayersFromTeam { players: player_msgs };

        let res2 =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr, &remove_players_msg, &[]);
        match res2 {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }


    #[test]
    fn add_new_remove_old_member() {
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        // add a new one and remove existing one
        let add = vec![Member {
            addr: USER3.into(),
            weight: 15,
        }];
        let remove = vec![USER1.into()];

        // non-admin cannot update
        let height = mock_env().block.height;
        let err = update_members(
            deps.as_mut(),
            height + 5,
            Addr::unchecked(USER1),
            add.clone(),
            remove.clone(),
        )
            .unwrap_err();
        assert_eq!(err, AdminError::NotAdmin {}.into());

        // Test the values from instantiate
        assert_users(&deps, Some(11), Some(6), None, None);
        // Note all values were set at height, the beginning of that block was all None
        assert_users(&deps, None, None, None, Some(height));
        // This will get us the values at the start of the block after instantiate (expected initial values)
        assert_users(&deps, Some(11), Some(6), None, Some(height + 1));

        // admin updates properly
        update_members(
            deps.as_mut(),
            height + 10,
            Addr::unchecked(INIT_ADMIN),
            add,
            remove,
        )
            .unwrap();

        // updated properly
        assert_users(&deps, None, Some(6), Some(15), None);

        // snapshot still shows old value
        assert_users(&deps, Some(11), Some(6), None, Some(height + 1));
    }

    #[test]
    fn add_old_remove_new_member() {
        // add will over-write and remove have no effect
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        // add a new one and remove existing one
        let add = vec![Member {
            addr: USER1.into(),
            weight: 4,
        }];
        let remove = vec![USER3.into()];

        // admin updates properly
        let height = mock_env().block.height;
        update_members(
            deps.as_mut(),
            height,
            Addr::unchecked(INIT_ADMIN),
            add,
            remove,
        ).unwrap();
        assert_users(&deps, Some(4), Some(6), None, None);
    }

    #[test]
    fn add_and_remove_same_member() {
        // add will over-write and remove have no effect
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        // USER1 is updated and remove in the same call, we should remove this an add member3
        let add = vec![
            Member {
                addr: USER1.into(),
                weight: 20,
            },
            Member {
                addr: USER3.into(),
                weight: 5,
            },
        ];
        let remove = vec![USER1.into()];

        // admin updates properly
        let height = mock_env().block.height;
        update_members(
            deps.as_mut(),
            height,
            Addr::unchecked(INIT_ADMIN),
            add,
            remove,
        )
            .unwrap();
        assert_users(&deps, None, Some(6), Some(5), None);
    }

    #[test]
    fn add_remove_hooks() {
        // add will over-write and remove have no effect
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        let hooks = team::state::HOOKS.query_hooks(deps.as_ref()).unwrap();
        assert!(hooks.hooks.is_empty());

        let contract1 = String::from("hook1");
        let contract2 = String::from("hook2");

        let add_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks {
            group_admin_hooks_msg: AddHook { addr: contract1.clone() },
        }};

        // non-admin cannot add hook
        let user_info = mock_info(USER1, &[]);
        let err = team::contract::execute(
            deps.as_mut(),
            mock_env(),
            user_info.clone(),
            add_msg.clone(),
        ).unwrap_err();

        assert_eq!(err,  TeamError::ManagableServiceError{ 0: ManagedServiceError::GroupAdminHooksError{ 0: GroupAdminError::HookError(HookError:: Admin(AdminError::NotAdmin {}))}}.into() );

        // admin can add it, and it appears in the query
        let admin_info = mock_info(INIT_ADMIN, &[]);
        let _ = team::contract::execute(
            deps.as_mut(),
            mock_env(),
            admin_info.clone(),
            add_msg.clone(),
        )
            .unwrap();
        let hooks = team::state::HOOKS.query_hooks(deps.as_ref()).unwrap();
        assert_eq!(hooks.hooks, vec![contract1.clone()]);

        // cannot remove a non-registered contract
        let remove_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks {
            group_admin_hooks_msg: RemoveHook {
                addr: contract2.clone(),
            }
        }};
        let err = team::contract::execute(deps.as_mut(), mock_env(), admin_info.clone(), remove_msg).unwrap_err();


        assert_eq!(err,   TeamError::ManagableServiceError{ 0:  ManagedServiceError::GroupAdminHooksError{ 0: GroupAdminError::HookError{ 0:  HookError::HookNotRegistered {}} } }.into());

        // add second contract
        let add_msg2 = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks {
            group_admin_hooks_msg: AddHook {
                addr: contract2.clone(),
            }
        }};
        let _ = team::contract::execute(deps.as_mut(), mock_env(), admin_info.clone(), add_msg2).unwrap();
        let hooks = team::state::HOOKS.query_hooks(deps.as_ref()).unwrap();
        assert_eq!(hooks.hooks, vec![contract1.clone(), contract2.clone()]);

        // cannot re-add an existing contract
        let err = team::contract::execute(deps.as_mut(), mock_env(), admin_info.clone(), add_msg).unwrap_err();

        assert_eq!(err,  TeamError::ManagableServiceError{
                                                            0: ManagedServiceError::GroupAdminHooksError{
                                                                0: GroupAdminError::HookError{
                                                                    0: HookError::HookAlreadyRegistered {}
                                                                }
                                                            }
                                                         }.into());


        // non-admin cannot remove
        let remove_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks { group_admin_hooks_msg: RemoveHook { addr: contract1 } }};
        let err = team::contract::execute(deps.as_mut(), mock_env(), user_info, remove_msg.clone()).unwrap_err();
        assert_eq!(err, TeamError::ManagableServiceError{
                                                            0: ManagedServiceError::GroupAdminHooksError{
                                                                0: GroupAdminError::HookError{
                                                                    0: HookError::Admin{
                                                                        0: AdminError::NotAdmin {}
                                                                    }
                                                                }

            }}.into());

                        /*TeamError::GroupAdminHooksError { 0: GroupAdminError::HookError(HookError::Admin(AdminError::NotAdmin {})) }.into());*/


        // remove the original
        let _ = team::contract::execute(deps.as_mut(), mock_env(), admin_info, remove_msg).unwrap();
        let hooks = team::state::HOOKS.query_hooks(deps.as_ref()).unwrap();
        assert_eq!(hooks.hooks, vec![contract2]);
    }

    #[test]
    fn hooks_fire() {
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        let hooks = team::state::HOOKS.query_hooks(deps.as_ref()).unwrap();
        assert!(hooks.hooks.is_empty());

        let contract1 = String::from("hook1");
        let contract2 = String::from("hook2");

        // register 2 hooks
        let admin_info = mock_info(INIT_ADMIN, &[]);
        let add_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks {
            group_admin_hooks_msg: AddHook {
                addr: contract1.clone(),
            }
        }};

        let add_msg2 = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks {
            group_admin_hooks_msg: AddHook {
                addr: contract2.clone(),
            }
        }};
        for msg in vec![add_msg, add_msg2] {
            let _ = team::contract::execute(deps.as_mut(), mock_env(), admin_info.clone(), msg).unwrap();
        }

        // make some changes - add 3, remove 2, and update 1
        // USER1 is updated and remove in the same call, we should remove this an add member3
        let add = vec![
            Member {
                addr: USER1.into(),
                weight: 20,
            },
            Member {
                addr: USER3.into(),
                weight: 5,
            },
        ];
        let remove = vec![USER2.into()];
        let msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::GroupAdminHooks { group_admin_hooks_msg: UpdateMembers { remove, add } }};

        // admin updates properly
        assert_users(&deps, Some(11), Some(6), None, None);
        let res = team::contract::execute(deps.as_mut(), mock_env(), admin_info, msg).unwrap();
        assert_users(&deps, Some(20), None, Some(5), None);

        // ensure 2 messages for the 2 hooks
        assert_eq!(res.messages.len(), 2);
        // same order as in the message (adds first, then remove)
        let diffs = vec![
            MemberDiff::new(USER1, Some(11), Some(20)),
            MemberDiff::new(USER3, None, Some(5)),
            MemberDiff::new(USER2, Some(6), None),
        ];
        let hook_msg = MemberChangedHookMsg { diffs };
        let msg1 = SubMsg::new(hook_msg.clone().into_cosmos_msg(contract1).unwrap());
        let msg2 = SubMsg::new(hook_msg.into_cosmos_msg(contract2).unwrap());
        assert_eq!(res.messages, vec![msg1, msg2]);
    }

    #[test]
    fn raw_queries_work() {
        // add will over-write and remove have no effect
        let mut deps = mock_dependencies();
        do_instantiate_team(deps.as_mut(), false, None, None);

        // get total from raw key
        let total_raw = deps.storage.get(TOTAL_KEY.as_bytes()).unwrap();
        let total: u64 = from_slice(&total_raw).unwrap();
        assert_eq!(17, total);

        // get member votes from raw key
        let member2_raw = deps.storage.get(&member_key(USER2)).unwrap();
        let member2: u64 = from_slice(&member2_raw).unwrap();
        assert_eq!(6, member2);

        // and execute misses
        let member3_raw = deps.storage.get(&member_key(USER3));
        assert_eq!(None, member3_raw);
    }


    #[test]
    fn add_all_players_to_team() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
        //let add_managed_contract_msg = AddManagedContract { asset_name: Some("Team #3".to_string()), asset_owner: Addr::unchecked(OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };
        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
        //                                                        goi_manager_addr.clone(), &add_managed_contract_msg, &[]);
        let players = all_players(goi_manager_addr);
        let player_msgs =
            build_player_contracts(&mut app, players, OWNER).unwrap();
        let players_msg = AddPlayersToTeam { players: player_msgs };

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr, &players_msg, &[]);
        match res {
            Ok(_) => {
                assert!(true)
            },
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn add_players_with_duplicate_name_to_team() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();
        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
        //let add_managed_contract_msg = AddManagedContract { asset_name:  Some("Team #7".to_string()), asset_owner: Addr::unchecked(OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };
        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
        //                                                        goi_manager_addr.clone(), &add_managed_contract_msg, &[]);
        let players = all_players_with_duplicate_name(goi_manager_addr);
        let player_msgs =
            build_player_contracts(&mut app, players, OWNER).unwrap();
        let players_msg = AddPlayersToTeam { players: player_msgs };

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr, &players_msg, &[]);
        match res {
            Ok(_) => {
                assert!(false)
            },
            Err(_) => {
                assert!(true)
            },
        };
    }

    #[test]
    fn add_players_with_duplicate_nams_across_teams() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();

        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
        //let add_managed_contract_msg = AddManagedContract { asset_name: Some("Team #4".to_string()), asset_owner: Addr::unchecked(OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };
        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
        //                                                        goi_manager_addr.clone(), &add_managed_contract_msg, &[]);
        let players = vec![
            get_player_instantiate_msg("same".to_string(), "name".to_string(),
                                       Positions::S, goi_manager_addr.clone()),
            get_player_instantiate_msg("CB!".to_string(), "Hash".to_string(),
                                       Positions::CB1, goi_manager_addr.clone())];


        let player_msgs =
            build_player_contracts(&mut app, players, OWNER).unwrap();
        let players_msg = AddPlayersToTeam { players: player_msgs };

        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &players_msg, &[]);

        let player = vec![
            get_player_instantiate_msg("same".to_string(), "name".to_string(),
                                       Positions::RB, goi_manager_addr.clone())];

        let res =
            build_player_contracts(&mut app, player, OWNER);

        match res {
            Ok(_) => {
                assert!(false)
            },
            Err(_) => {
                assert!(true)
            },
        };

    }

    #[test]
    //guard against attempts to init players on fake goi_manager contract
    // and then add to legitimate goi_manager contracts
    fn add_players_to_teams_across_different_managing_contracts() {
        let mut app = mock_app(&[]);
        let goi_manager_addr =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();

        let goi_manager_addr_2 =
            instantiate_management_contract_with_user
                (&mut app, OWNER.to_string()).unwrap();

        let team_addr =
            instantiate_team_with_managed_contract
                (&mut app, vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));
       // let add_managed_contract_msg = AddManagedContract { asset_name: Some("Team #5".to_string()), asset_owner: Addr::unchecked(OWNER), contract_address: team_addr.clone(), contract_type: AssetTypes::Team };

        //let add_managed_contract_msg_res = app.execute_contract(Addr::unchecked(OWNER),
         //                                                       goi_manager_addr.clone(), &add_managed_contract_msg, &[]);




        let players = vec![
            get_player_instantiate_msg("player1".to_string(), "name".to_string(),
                                       Positions::S, goi_manager_addr.clone()),
            get_player_instantiate_msg("CB!".to_string(), "Hash".to_string(),
                                       Positions::CB1, goi_manager_addr.clone())];


        let player_msgs =
            build_player_contracts(&mut app, players, OWNER).unwrap();

        let players_msg = AddPlayersToTeam { players: player_msgs };


        let res =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr.clone(), &players_msg, &[]);



        let player = vec![
            get_player_instantiate_msg("player2".to_string(), "name".to_string(),
                                       Positions::RB, goi_manager_addr_2.clone())];

        let player_msgs_2 =
            build_player_contracts(&mut app, player, OWNER).unwrap();

        //execution should generate  TeamError::ErrorCreatingPlayer erro
        let res_2 =
            app.execute_contract(Addr::unchecked(OWNER),
                                 team_addr, &player_msgs_2, &[]);

        match res_2 {
            Ok(_) => {
                assert!(false)
            },
            Err(_) => {
                assert!(true)
            }
        };
    }


}
