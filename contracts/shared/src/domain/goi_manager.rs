use std::collections::HashMap;

use cosmwasm_std::{Addr, Coin, CosmosMsg, ReplyOn, Response, SubMsg, Timestamp, to_binary, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::AppInfo;
use crate::display::DisplayInfo;
use crate::goi_manager::ExecuteMsg::{AddManagedContract, ManagedAssetSoldHook, UpdateAssetForSaleStatusHook};
use crate::league::LeagueInfo;
use crate::manage::{ManagedStatusChangedHookMsg, ManagementFee};
use crate::player::PlayerInfo;
use crate::rewards::Reward;
use crate::team::TeamInfo;
use crate::utils::general::AssetTypes;
use group_admin::messages::receive::ExecuteMsg as GroupAdminHooksMsg;
use crate::goi_manager;
use crate::messaging::{JoinSeasonRequestInfo, MessageTypes};
use crate::season::Season;
use crate::utils::{MessageId, LeagueAddr, SeasonId, TeamAddr};


pub fn get_minters() -> Vec<Addr> {
    vec![
        //TODO: Retrieve this list from a minters contract
        Addr::unchecked("creator"),
        Addr::unchecked("funny"),
        Addr::unchecked("somebody"),
        Addr::unchecked("else"),
        Addr::unchecked("admin0001"),

    ]
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GoiMangerContractModel {
    pub owner: Addr,

    pub teams: Option< HashMap<Addr, TeamInfo>>,
    pub leagues: Option< HashMap<Addr, LeagueInfo>>,
    pub displays: Option< HashMap<Addr, DisplayInfo>>,
    pub apps:Option< HashMap<Addr, AppInfo>>,

    pub rewards: Option<HashMap<Addr, Vec<Reward>>>,
    pub players: Option<Vec<PlayerInfo>>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoiManagerQueryMsg {
    GetPlayerByName{first_name: String, last_name: String},
    GetManagedContract { contract_address: Addr, contract_type: AssetTypes},
    ManagementQryMessages { management_qry_msg: ManagementQryMsg},
    GetOwnerAssets{ owner_address: Addr},
    GetAssetsForSale{ contract_type: AssetTypes },
    GetAllSeasonsForLeague { league_address: Addr},
    GetActiveSeasonsForLeague  { league_address: Addr},
    GetUpcomingSeasonsForLeague { league_address: Addr},
    GetPastSeasonsForLeague{league_address: Addr},
    GetSeasonById { season_id: u64},
    GetUpComingSeasonsForAllLeagues {},
    CheckSeasonDateRangeForLeague { start_date: Timestamp, end_date: Timestamp, league_addr: Addr },
    GetMessagesToItem { item_addr: Addr, asset_type: AssetTypes },
    GetMessagesFromItem {  item_addr: Addr, asset_type: AssetTypes },
    GetLeagueTeams { league_addr: LeagueAddr}
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManagementQryMsg {
    GetManagementInfo {},
    GetManagedContract{contract: Addr}
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ManagedStatusChangedHook (ManagedStatusChangedHookMsg),
    AddManagedContract { asset_name: Option<String>, asset_owner: Addr, contract_type: AssetTypes},
    GroupAdminHooks {group_admin_hooks_msg: GroupAdminHooksMsg },
    UpdateFees { add: Option<Vec<ManagementFee>>, remove: Option<Vec<i32>>},
    AddPlayersToTeam { players: Vec<PlayerInfo>},
    UpdateAssetForSaleStatusHook { for_sale_status: bool, price: Option<Coin> },
    ManagedAssetSoldHook { new_owner: Addr },
    Withdraw{ recipient: Addr, amount: Vec<Coin> },
    AddSeasonToLeague { season: Season },
    //Sending (user, not contract) must own teams being
    //added to league
    AddTeamsToLeague { teams: Vec<TeamAddr>, sending_user: Addr},
    UpdateSeasonStatus { season_id: SeasonId, status: MessageTypes },
    JoinLeague { season_id: SeasonId },
    CancelSeasonSpot { season_id: SeasonId},
    JoinLeagueWinnerTakeAll { season_id: SeasonId, fee: Vec<Coin> },
}





pub fn on_successful_init_processing(asset_name: Option<String>, asset_owner: Addr, asset_type: AssetTypes, managing_contract: Addr, response: Response) ->  Response {
    let add_managed_contract_msg =
        AddManagedContract {
            asset_name,
            asset_owner,
            contract_type: asset_type
        };
    let exc_msg: CosmosMsg =
        WasmMsg::Execute {
            contract_addr: managing_contract.to_string(),
            msg: to_binary(&add_managed_contract_msg).expect("Expected known add_managed_contract_msg msg"),
            funds: vec![]
        }.into();

    let res_sub_msg =
        SubMsg {
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}

pub fn on_successful_forsale_update(for_sale_status: bool, price: Option<Coin>, managing_contract: Addr, response: Response) ->  Response  {
    let for_sale_hook_status_update =
       UpdateAssetForSaleStatusHook  {  for_sale_status, price };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&for_sale_hook_status_update).expect("Expected known for_sale_hook_status_update msg") , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}


pub fn on_successful_buy(new_owner: Addr, managing_contract: Addr, response: Response) ->  Response {
    let asset_sold_hook_msg = ManagedAssetSoldHook { new_owner };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&asset_sold_hook_msg).expect("Expected known asset_sold_hook_msg msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}


pub fn send_add_season_msg_to_goi_manager(season: Season,  managing_contract: Addr, response: Response) ->  Response {
    let add_season_msg =
        goi_manager::ExecuteMsg::AddSeasonToLeague { season: season };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&add_season_msg).expect("Expected known add_season_msg msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}

pub fn send_add_team_to_league_msg_to_goi_manager(teams: Vec<TeamAddr>, sending_user: Addr,   managing_contract: Addr,  response: Response)  -> Response {
    let add_teams_msg =
        goi_manager::ExecuteMsg::AddTeamsToLeague { teams, sending_user };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&add_teams_msg).expect("Expected known add_teams msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}






pub fn update_messaging_item_msg_to_goi_manager(invite_id: MessageId, updated_invite_message_status: MessageTypes, managing_contract: Addr, response: Response) -> Response {
    let update_msg =
        goi_manager::ExecuteMsg::UpdateSeasonStatus {
            season_id: invite_id,
            status: updated_invite_message_status
        };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&update_msg).expect("Expected known update invite_message msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}

pub fn send_request_to_join_open_season(season_id: SeasonId, managing_contract: Addr, response: Response) -> Response {
    let join_league_msg =
        goi_manager::ExecuteMsg::JoinLeague {
            season_id
        };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&join_league_msg).expect("Expected known join_league msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}


pub fn send_request_to_cancel_season_spot(season_id: SeasonId, managing_contract: Addr, response: Response) -> Response {
    let msg =
        goi_manager::ExecuteMsg::CancelSeasonSpot {
            season_id
        };
    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&msg).expect("Expected known join_league msg")  , funds: vec![] }.into();

    let res_sub_msg =
        SubMsg{
            id: 0,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessage(res_sub_msg)
}



pub fn send_request_to_join_winner_takes_all_season(season_id: SeasonId, fee: Vec<Coin>, managing_contract: Addr, response: Response) -> Response {
    let res:CosmosMsg =
        cosmwasm_std::BankMsg::Send
        {
            to_address: managing_contract.to_string(),
            amount: fee.clone()
        }.into();
    let res_fee_sub_msg =
        SubMsg{
            id: 0,
            msg: res,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };


    let join_league_msg =
        goi_manager::ExecuteMsg::JoinLeagueWinnerTakeAll {
            season_id,
            fee: fee.clone(),
        };

    let exc_msg:CosmosMsg =
        WasmMsg::Execute { contract_addr: managing_contract.to_string(),
            msg: to_binary(&join_league_msg).expect("Expected known join_league msg"), funds: fee }.into();

    let res_join_league_sub_msg =
        SubMsg{
            id: 1,
            msg: exc_msg,
            gas_limit: None,
            reply_on: ReplyOn::Never
        };
    response.add_submessages( vec![res_fee_sub_msg, res_join_league_sub_msg])
}
