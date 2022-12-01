use std::borrow::BorrowMut;
use anyhow::Error;
use cosmwasm_std::{Addr, Api, Coin, DepsMut, Empty, OwnedDeps, Querier, StdResult, Storage, Timestamp, Uint128};
use cosmwasm_std::testing::{mock_env, mock_info};
use cw4::Member;
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
use cw4_group::state::MEMBERS;
use group_admin::service::list_members;
use league::msg::ExecuteMsg::{AddSeasonToLeague, AddTeamsToLeague};
use league::msg::InstantiateLeagueMsg;
use managed::messages::ManagedExecuteMsg;
use saleable::messages::receive::ExecuteMsg::{Buy, Update};
use shared::player::{InstantiateMsg, PlayerInfo};
use shared::player_attributes::{AttrItem, PlayerAttributes, Positions};
use shared::player::{InstantiateMsg as plIntantiateMsg};
use shared::utils::{LeagueAddr, MessageId, SeasonId, TeamAddr, MAX_TEAMS_ALLOWED};
use team::msg::ExecuteMsg::{ManagedServiceMessage, JoinLeague};
use team::msg::InstantiateTeamMsg;
use team::TeamError;
use anyhow::Result as AnyResult;
use shared::data::ModelItem;
use shared::goi_manager::GoiManagerQueryMsg::GetMessagesToItem;
use shared::messaging::{JoinSeasonRequestInfo, Message, MessageTypes};
use shared::season::{Season, SeasonAccessTypes, SeasonModelData, SeasonStatus};
use shared::utils::general::AssetTypes;


pub const OWNER: &str = "admin0001";
pub const INIT_ADMIN: &str = "juan";
pub const USER1: &str = "somebody";
pub const USER2: &str = "else";
pub const USER3: &str = "funny";
pub const TOKEN: &str = "ujuno";

pub const CHAIN_ID: &str = "TEST-01";
pub const COIN_DENOM: &str = "ABC";



//Offensive players
pub fn get_player_instantiate_msg(first_name: String, last_name: String,
                              position: Positions, managing_contract_address: Addr) -> plIntantiateMsg {
    plIntantiateMsg {
        first_name,
        last_name,
        position,
        attributes: PlayerAttributes {
            hands: AttrItem { value: "0.0".to_string() },
            accuracy: AttrItem { value: "0.0".to_string() },
            speed: AttrItem { value: "0.0".to_string() },
            strength: AttrItem { value: "0.0".to_string() },
            leader: AttrItem { value: "0.0".to_string() },
            pressure_threshold: AttrItem { value: "0.0".to_string() },
            agility: AttrItem { value: "0.0".to_string() },
            football_iq: AttrItem { value: "0.0".to_string() },
            temperament: AttrItem { value: "0.0".to_string() },
            angle_of_view: 0
        },
        managing_contract_address
    }
}


pub fn all_players(managing_contract_address: Addr) -> Vec<plIntantiateMsg> {
    vec![
        //Defense
        get_player_instantiate_msg("S".to_string(), "Hash".to_string(),
                                   Positions::S, managing_contract_address.clone()),
        get_player_instantiate_msg("CB!".to_string(), "Hash".to_string(),
                                   Positions::CB1, managing_contract_address.clone()),
        get_player_instantiate_msg("CB2".to_string(), "Hash".to_string(),
                                   Positions::CB2, managing_contract_address.clone()),
        get_player_instantiate_msg("LB".to_string(), "Hash".to_string(),
                                   Positions::LB, managing_contract_address.clone()),
        get_player_instantiate_msg("TR".to_string(), "Hash".to_string(),
                                   Positions::TR, managing_contract_address.clone()),
        get_player_instantiate_msg("TL".to_string(), "Hash".to_string(),
                                   Positions::TL, managing_contract_address.clone()),
        get_player_instantiate_msg("CD".to_string(), "Hash".to_string(),
                                   Positions::CD, managing_contract_address.clone()),
        //Offense
        get_player_instantiate_msg("QB".to_string(), "Hash".to_string(),
                                   Positions::QB, managing_contract_address.clone()),
        get_player_instantiate_msg("RB".to_string(), "Hash".to_string(),
                                   Positions::RB, managing_contract_address.clone()),
        get_player_instantiate_msg("GR".to_string(), "Hash".to_string(),
                                   Positions::GR, managing_contract_address.clone()),
        get_player_instantiate_msg("GL".to_string(), "Hash".to_string(),
                                   Positions::GL, managing_contract_address.clone()),
        get_player_instantiate_msg("WR1".to_string(), "Hash".to_string(),
                                   Positions::WR1, managing_contract_address.clone()),
        get_player_instantiate_msg("WR2".to_string(), "Hash".to_string(),
                                   Positions::WR2, managing_contract_address.clone()),
        get_player_instantiate_msg("CO".to_string(), "Hash".to_string(),
                                   Positions::CO, managing_contract_address.clone()),
    ]
}

pub fn all_players_with_duplicate_name(managing_contract_address: Addr) -> Vec<plIntantiateMsg> {
    vec![
        //Defense
        get_player_instantiate_msg("S".to_string(), "Hash".to_string(),
                                   Positions::S, managing_contract_address.clone()),
        get_player_instantiate_msg("CB!".to_string(), "Hash".to_string(),
                                   Positions::CB1, managing_contract_address.clone()),
        get_player_instantiate_msg("CB2".to_string(), "Hash".to_string(),
                                   Positions::CB2, managing_contract_address.clone()),
        get_player_instantiate_msg("LB".to_string(), "Hash".to_string(),
                                   Positions::LB, managing_contract_address.clone()),
        get_player_instantiate_msg("TR".to_string(), "Hash".to_string(),
                                   Positions::TR, managing_contract_address.clone()),
        get_player_instantiate_msg("TL".to_string(), "Hash".to_string(),
                                   Positions::TL, managing_contract_address.clone()),
        get_player_instantiate_msg("CD".to_string(), "Hash".to_string(),
                                   Positions::CD, managing_contract_address.clone()),
        //Offense
        get_player_instantiate_msg("QB".to_string(), "Hash".to_string(),
                                   Positions::QB, managing_contract_address.clone()),
        get_player_instantiate_msg("QB".to_string(), "Hash".to_string(),
                                   Positions::RB, managing_contract_address.clone()),
        get_player_instantiate_msg("GR".to_string(), "Hash".to_string(),
                                   Positions::GR, managing_contract_address.clone()),
        get_player_instantiate_msg("GL".to_string(), "Hash".to_string(),
                                   Positions::GL, managing_contract_address.clone()),
        get_player_instantiate_msg("WR1".to_string(), "Hash".to_string(),
                                   Positions::WR1, managing_contract_address.clone()),
        get_player_instantiate_msg("WR2".to_string(), "Hash".to_string(),
                                   Positions::WR2, managing_contract_address.clone()),
        get_player_instantiate_msg("CO".to_string(), "Hash".to_string(),
                                   Positions::CO, managing_contract_address.clone()),
    ]
}




pub fn contract_player() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        player::contract::instantiate,
        player::contract::instantiate,
        player::contract::query,
    );
    Box::new(contract)
}



pub fn contract_team() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        team::contract::execute,
        team::contract::instantiate,
        team::contract::query,
    );
    Box::new(contract)
}


pub fn contract_league() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        league::contract::execute,
        league::contract::instantiate,
        league::contract::query,
    );
    Box::new(contract)
}





// uploads code and returns address of team contract
pub fn instantiate_league_with_managed_contract(app: &mut App, admin: String, members: Vec<Member>,
                                              manager_contract_address: Option<Addr>) -> Addr {
    let league_id = app.store_code(contract_league());
    let msg = league::msg::InstantiateLeagueMsg {
        name: "Big Bang League".to_string(),
        admin,
        members,
        managing_contract: manager_contract_address,
        for_sale: false,
        price: None
    };
    app.instantiate_contract
    (league_id, Addr::unchecked(OWNER), &msg,
     &[], "League", None)
        .unwrap()
}



// uploads code and returns address of team contract
pub fn instantiate_team_with_managed_contract(app: &mut App, members: Vec<Member>,
                                              manager_contract_address: Option<Addr>) -> Addr {
    let group_id = app.store_code(contract_team());
    let msg = team::msg::InstantiateTeamMsg {
        name: "Big Bang".to_string(),
        admin: OWNER.into(),
        members,
        managing_contract: manager_contract_address,
        for_sale: false,
        price: None
    };
    app.instantiate_contract
    (group_id, Addr::unchecked(OWNER), &msg,
     &[], "team", None)
        .unwrap()
}


pub fn instantiate_team_with_managed_contract_with_sender_admin(app: &mut App, user: &str, members: Vec<Member>,
                                                                manager_contract_address: Option<Addr>) -> Addr {
    let group_id = app.store_code(contract_team());
    let msg = team::msg::InstantiateTeamMsg {
        name: "Big Bang".to_string(),
        admin: user.into(),
        members,
        managing_contract: manager_contract_address,
        for_sale: false,
        price: None
    };
    app.instantiate_contract
    (group_id, Addr::unchecked(user), &msg,
     &[], "team", None)
        .unwrap()
}



// uploads code and returns address of team contract
pub fn instantiate_player(app: &mut App, msg: plIntantiateMsg, sender: &str) -> anyhow::Result<Addr> {
    let player_id = app.store_code(contract_player());
    let l_name = String::from(msg.first_name.clone());
    let space = &" ".to_string();
    let f_name = &String::from(msg.last_name.clone());
    app.instantiate_contract
    (player_id, Addr::unchecked(sender),
     &msg, &[], f_name.to_owned() + space + &l_name, None)

}

pub fn build_player_contracts
(app: &mut App, players_instantiation_msgs: Vec<plIntantiateMsg>, sender: &str)
 -> Result<Vec<PlayerInfo>, TeamError> {
    let mut res_hold: Vec<PlayerInfo> = Vec::default();
    let mut init_error: Option<InstantiateMsg> = None;
    for a_msg in players_instantiation_msgs {
        let player_address_res = instantiate_player(app, a_msg.clone(), sender);
        match player_address_res {
            Ok(add) => {
                res_hold.push(PlayerInfo
                { first_name: a_msg.first_name, last_name: a_msg.last_name, address: add, position: a_msg.position, assigned_team_address: None })
            }
            Err(_) => {
                init_error = Some( a_msg);
                break;
            }
        }

    }
    match init_error  {
        Some(m) => {
            Err(TeamError::ErrorCreatingPlayer { msg: m })
        }
        None => {
            Ok(res_hold)
        }
    }
}

pub fn team_request_to_join_league(app: &mut App, user: &str, team_addr: TeamAddr, season_id: SeasonId) -> anyhow::Result<AppResponse> {
    let team_request_to_join_league = JoinLeague {  season_id };
    app.execute_contract(Addr::unchecked(user),
                         team_addr.clone(),
                         &team_request_to_join_league, &[])
}



pub fn get_messages(app: &mut App, managing_contract_addr: Addr, target_asset_addr: Addr, target_asset_type: AssetTypes)
        -> Option<Vec<Message<JoinSeasonRequestInfo>>>{
     app
        .wrap()
        .query_wasm_smart(
            &managing_contract_addr.clone(),
            &GetMessagesToItem {
                item_addr: target_asset_addr,
                asset_type: target_asset_type
            })
        .unwrap()
}







pub fn update_message_status(app: &mut App, user: &str,  target_contract_addr: Addr, target_contract_asset_type: AssetTypes, message_id: MessageId, update_message: MessageTypes) -> anyhow::Result<AppResponse> {
        match target_contract_asset_type {
            AssetTypes::Team => {
                app.execute_contract(Addr::unchecked(user),
                                     target_contract_addr,
                                     & team::msg::ExecuteMsg::UpdateMessageStatus
                                            {  message_id,
                                               updated_message_status: update_message
                                            },
                                     &[])
            }
            AssetTypes::League => {
                app.execute_contract(Addr::unchecked(user),
                                     target_contract_addr,
                                     &league::msg::ExecuteMsg::UpdateMessageStatus
                                            {
                                                message_id,
                                                updated_message_status: update_message
                                            },
                                     &[])
            }
            AssetTypes::Display => {
                panic!("Type not supported!")
            }
            AssetTypes::App => {
                panic!("Type not supported!")
            }
        }
}

pub fn add_season_to_league(app: &mut App, user: &str, season: SeasonModelData, league_addr: LeagueAddr)
                            -> anyhow::Result<AppResponse> {
    let add_season_to_league_msg =
            AddSeasonToLeague { season_name: "Big House".to_string(), season_model: season };

   
    app.execute_contract(Addr::unchecked(user),
    league_addr.clone(),
                        &add_season_to_league_msg, &[])
}



pub fn get_season_with_custom_settings(season_id: u64, access_type: SeasonAccessTypes,
                                       start_date: Timestamp, end_date: Timestamp) -> SeasonModelData {
    let res =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("season ".to_string() + &season_id.to_string()), },
            start_date: ModelItem { update: true, data: Some(start_date) },
            end_date: ModelItem { update: true, data: Some( end_date ) },
            access_type: ModelItem { update: true, data: Some(access_type) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED) },

        };

    res
}

pub fn get_season(season_id: u64, start_date: Timestamp, end_date: Timestamp) -> SeasonModelData {
    let res =
        SeasonModelData{
            description: ModelItem { update: true, data: Some("season ".to_string() + &season_id.to_string()), },
            start_date: ModelItem { update: true, data: Some(start_date) },
            end_date: ModelItem { update: true, data: Some( end_date ) },
            access_type: ModelItem { update: true, data: Some(SeasonAccessTypes::Open) },
            status: ModelItem { update: true, data: Some(SeasonStatus::Active) },
            max_teams_allowed: ModelItem { update: true, data: Some(MAX_TEAMS_ALLOWED) },
        };
    
    res
}


pub fn get_team_for_users(users: Vec<Addr>, goi_manager_addr: Addr, app: &mut App) -> Vec<TeamAddr> {
    let mut teams: Vec<Addr> = vec![];
    for a_user in users {
        let team_addr = instantiate_team_with_managed_contract
            (app , vec![member(OWNER, 100)], Some(goi_manager_addr.clone()));

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
                             team_addr.clone(), &for_sale_msg, &[]).unwrap();

        let buy_team_msg = ManagedServiceMessage { message: ManagedExecuteMsg::Saleable { saleable_msg: Buy {} } };



        app.execute_contract(a_user,
                             team_addr.clone(), &buy_team_msg,
                             &[team_sell_price.clone()]).unwrap();

        teams.push(team_addr);
    }
    teams
}


pub fn get_league_for_users(users: Vec<Addr>, goi_manager_addr: Addr,app: &mut App) -> Vec<LeagueAddr> {
    let mut leagues: Vec<Addr> = vec![];
    for a_user in users {
        let league_addr = instantiate_league_with_managed_contract(app, OWNER.clone().to_string(),
                                                               vec![Member { addr: OWNER.clone().to_string(), weight: 100 }], Some(goi_manager_addr.clone()));

        let league_sell_price = Coin { denom: TOKEN.to_string(), amount: Uint128::from(500000u128) };
        let for_sale_msg =
            ManagedServiceMessage {
                message: ManagedExecuteMsg::Saleable {
                    saleable_msg: Update {
                        for_sale_status: true,
                        price: Some(league_sell_price.clone())
                    }
                }
            };


        app.execute_contract(Addr::unchecked(OWNER),
                             league_addr.clone(), &for_sale_msg, &[]).unwrap();

        let buy_league_msg = ManagedServiceMessage { message: ManagedExecuteMsg::Saleable { saleable_msg: Buy {} } };



        app.execute_contract(a_user,
                             league_addr.clone(), &buy_league_msg,
                             &[league_sell_price.clone()]).unwrap();

        leagues.push(league_addr);
    }
    leagues
}


pub fn do_instantiate_team(deps: DepsMut, for_sale: bool, price: Option<Coin>,
                       managing_contract: Option<Addr>) {
    let msg = InstantiateTeamMsg {
        name: "Power Thatters".to_string(),
        admin: INIT_ADMIN.into(),
        members: vec![
            Member {
                addr: USER1.into(),
                weight: 11,
            },
            Member {
                addr: USER2.into(),
                weight: 6,
            },
        ],

        managing_contract,

        for_sale,
        price
    };
    let info = mock_info("creator", &[]);
    team::contract::instantiate(deps, mock_env(), info, msg).unwrap();
}

pub fn assert_users<S: Storage, A: Api, Q: Querier>(
    deps: &OwnedDeps<S, A, Q>,
    user1_weight: Option<u64>,
    user2_weight: Option<u64>,
    user3_weight: Option<u64>,
    height: Option<u64>,
) {
    let member1 =
        team::contract::query_member(deps.as_ref(), USER1.into(), height).unwrap();
    assert_eq!(member1.weight, user1_weight);

    let member2 =
        team::contract::query_member(deps.as_ref(), USER2.into(), height).unwrap();
    assert_eq!(member2.weight, user2_weight);

    let member3 =
        team::contract::query_member(deps.as_ref(), USER3.into(), height).unwrap();
    assert_eq!(member3.weight, user3_weight);

    // this is only valid if we are not doing a historical query
    if height.is_none() {
        // compute expected metrics
        let weights = vec![user1_weight, user2_weight, user3_weight];
        let sum: u64 = weights.iter().map(|x| x.unwrap_or_default()).sum();
        let count = weights.iter().filter(|x| x.is_some()).count();

        // TODO: more detailed compare?
        let members =
            list_members(deps.as_ref(), None, None, MEMBERS).unwrap();
        assert_eq!(count, members.members.len());

        let total = team::contract::query_total_weight(deps.as_ref()).unwrap();
        assert_eq!(sum, total.weight); // 17 - 11 + 15 = 21
    }
}



pub fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}


pub fn contract_management() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        goi_manager::contract::execute,
        goi_manager::contract::instantiate,
        goi_manager::contract::query,
    );
    Box::new(contract)
}

pub fn mock_app(init_funds: &[Coin]) -> App {
    mock_app_by_user(vec![OWNER], init_funds)
}

pub fn mock_app_by_user(users: Vec<&str>, init_funds: &[Coin]) -> App {
    AppBuilder::new().build(|router, _, storage, | {
        for a_user in users {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(a_user),
                              init_funds.to_vec())
                .unwrap();
        }
    })
}

pub fn instantiate_management_contract(app: &mut App) -> Addr {
    instantiate_management_contract_with_user
        (app.borrow_mut(), OWNER.to_string()).unwrap()
}


pub fn instantiate_management_contract_with_user(app: &mut App, user: String)
                                             -> Result<Addr, Error> {
    let code_id = app.store_code(contract_management());
    let msg = goi_manager::msg::InstantiateMsg {
        owner: Addr::unchecked(user.to_string()),
        admin: user.clone(),
        members: vec![],
        teams: None,
        teams_for_sale: None
    };
    app.instantiate_contract
    (code_id, Addr::unchecked(user.to_string()),
     &msg, &[], "goi_manager", None)
}

pub fn get_team_for_sale(app: &mut App, goi_manager_addr1: Addr, team_sell_price: Coin) -> Option<Addr>{

    let team_addr = instantiate_team_with_managed_contract
        (&mut app.borrow_mut(), vec![member(OWNER, 100)],
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
    match res {
        Ok(a) => {
            let resp = a;
            Some(team_addr)
        }
        Err(e) => {
            let Er = e;
            None
        }
    }


}


pub fn buy_team(app: &mut App, buyer_addr: Addr, team_address: Addr,  buy_amount_sent: Coin  )  {

    let buy_team_msg = ManagedServiceMessage{ message:  ManagedExecuteMsg::Saleable { saleable_msg: Buy {} }};
    app.execute_contract(buyer_addr,
                         team_address, &buy_team_msg,
                         &[buy_amount_sent]).unwrap();
}

pub mod apps {
    use anyhow::Error;
    use cosmwasm_std::{Addr, Coin, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use shared::utils::{BlockTime, JsonData};
    use shared::utils::xnodes::SuccessfulExecutionCount;

    use crate::shared_utils::{member, OWNER};

    fn contract_application() -> Box<dyn Contract<Empty>> {

        let contract = ContractWrapper::new(
            application::contract::execute,
            application::contract::instantiate,
            application::contract::query);

        let reply_contract =
            ContractWrapper::with_reply(contract,application::contract::reply);
        Box::new(reply_contract)
    }

    pub fn contract_task() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            task::contract::execute,
            task::contract::instantiate,
            task::contract::query,
        );
        Box::new(contract)
    }

    pub fn instantiate_task_contract
    (app: &mut App, target_executable_contact: Addr, bond_amount: Vec<Coin>,
     exec_msg: Option<JsonData>, start_date: BlockTime,
     end_date: Option<BlockTime>,reward_threshold: SuccessfulExecutionCount, task_id: u8)
     -> Result<Addr, Error> {
        instantiate_task_contract_by_owning_user(app, OWNER.to_string(),
                                                 target_executable_contact, bond_amount,
                                          exec_msg,start_date, end_date, reward_threshold, task_id)
    }

    pub fn instantiate_task_contract_by_owning_user
    (app: &mut App, owning_user: String, target_executable_contact: Addr, bond_amount: Vec<Coin>,
     exec_msg: Option<JsonData>, start_date: BlockTime,
     end_date: Option<BlockTime>,reward_threshold: SuccessfulExecutionCount, task_id: u8)
     -> Result<Addr, Error> {
        let code_id = app.store_code(contract_task());
        let msg = task::msg::InstantiateMsg {
            task_id,
            name: "the main task".to_string(),
            description: Some("This is a description!!".to_string()),
            admin: owning_user,
            start_date ,
            end_date,
            reward_threshold,
            bond_amount,
            exec_msg,
            target_executable_contact
        };
        app.instantiate_contract
        (code_id, Addr::unchecked(OWNER.to_string()),
         &msg, &[], "task", None)
    }

    pub fn instantiate_application_contract(app: &mut App, app_name: String,  managing_contract: Option<Addr>)  ->  Addr {
        instantiate_application_contract_with_user(app, OWNER.to_string(), app_name, managing_contract)
    }

    pub fn instantiate_application_contract_with_user
        (app: &mut App, user: String, app_name: String, managing_contract: Option<Addr>)
                                                     -> Addr {
        let code_id = app.store_code(contract_application());
        let msg = application::msg::InstantiateMsg {
            app_id: "appId-01".to_string(),
            app_name,
            admin: user.to_string(),
            members: vec![member(user, 100)],
            managing_contract,
            for_sale: false,
            price: None
        };
        app.instantiate_contract
        (code_id, Addr::unchecked(OWNER.to_string()),
         &msg, &[], "application", None).unwrap()
    }


}



pub fn add_owner_teams_to_league(app: &mut App, teams: Vec<TeamAddr>, league_addr: LeagueAddr, league_owner: Addr) -> AnyResult<AppResponse> {
    let add_team_to_league_msg = AddTeamsToLeague {
        team_addresses: teams
    };

    app.execute_contract(league_owner,
                     league_addr,
                     &add_team_to_league_msg, &[])

}