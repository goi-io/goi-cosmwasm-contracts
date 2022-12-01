use cosmwasm_std::{Addr, Coin, StdResult, Storage, Timestamp, Uint128};
use cw4::TOTAL_KEY;
use cw_controllers::{Admin, Hooks};
use cw_storage_plus::{Item, SnapshotMap, Strategy, Map, UniqueIndex, MultiIndex, IndexList, Index, IndexedMap, PrimaryKey, Key};
use schemars::_private::NoSerialize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use manager::service::ManagementService;

use shared::{player::{PlayerInfo, PlayerTrackingItem}, goi_manager::GoiMangerContractModel, utils::MangedItem};
use shared::league::{LeagueInfo, SeasonActiveStatus};
use shared::team::{ TeamInfo};

use shared::utils::general::{AssetTypes, merge_strings};
use shared::utils::{FName, PlayerAddr, TeamAddr, PlayerTeamAddr, TeamPlayerAddr, OwnerAddr, LNameFNameString, LName, ForSaleStatus, AssetSaleItems, LeagueAddr, SeasonId, SeasonActiveStatusValue, BlockChainTimeValue, StartDate, EndDate, MessageId, AsseTypes_u8, InviteAccepted, SeasonDepositId};

use shared::link_team_player::LinkTeamPlayer;
use shared::manage::ManagedStatus;
use shared::messaging::{JoinSeasonRequestInfo, Message};
use shared::player::PlayerInfoPacket;
use shared::season::{Season, SeasonLedger};



pub const PLAYER_NAMES: Item<PlayerInfoPacket> = Item::new("teams_players_names");
//pub const ASSETS_FOR_SALE: Item<AssetSaleItems> = Item::new("TEAMS_FOR_SALE");
pub const INDEX_COUNTER: Item<u64> = Item::new("index_counter");

pub fn next_index_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = INDEX_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    INDEX_COUNTER.save(store, &id)?;
    Ok(id)
}


// MANAGED_ASSETS
pub struct ManagedAssetIndexes<'a> {
    pub id: UniqueIndex<'a, Addr, MangedItem>,
    pub owner: MultiIndex<'a, Addr, MangedItem, Addr>,
    pub asset_type: MultiIndex<'a, (u8, Addr), MangedItem, Addr>,
    pub managed_status: MultiIndex<'a, (u8, Addr), MangedItem, Addr>,
    pub for_sale: MultiIndex<'a, (u8, Addr), MangedItem, Addr>
}

impl<'a> IndexList<MangedItem> for ManagedAssetIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<MangedItem>> + '_> {
        let v: Vec<&dyn Index<MangedItem>> = vec![ &self.owner, &self.for_sale, &self.asset_type, &self.managed_status];
        Box::new(v.into_iter())
    }
}

pub fn managed_assets<'a>() -> IndexedMap<'a, &'a  Addr, MangedItem, ManagedAssetIndexes<'a>> {
    let indexes = ManagedAssetIndexes {
        id: UniqueIndex::new(|d| d.clone().asset_addr, "ASSETS"),
        owner: MultiIndex::new(|t, key| key.clone().asset_owner, "ASSETS", "ASSET_OWNER"),
        asset_type:  MultiIndex::new(|t, key|
                                         {
                                             let type_val =
                                                    match key.clone().asset_type {
                                                        AssetTypes::Team => {
                                                            0u8
                                                        },
                                                        AssetTypes::League => {
                                                            1u8
                                                        },
                                                        AssetTypes::Display => {
                                                            2u8
                                                        },
                                                        AssetTypes::App => {
                                                            3u8
                                                        }
                                                    };
                                             (type_val, key.clone().asset_addr)

                                         }, "ASSETS", "ASSET_TYPE"),
        managed_status:  MultiIndex::new(|t, key|
                                             {
                                                 let key_val =
                                                        match key.clone().managed_status {
                                                            ManagedStatus::Pending => {
                                                                0u8
                                                            },
                                                            ManagedStatus::Enabled => {
                                                                1u8
                                                            },
                                                            ManagedStatus::Disabled => {
                                                                2u8
                                                            },
                                                            ManagedStatus::Suspended => {
                                                                3u8
                                                            }
                                                        };
                                                 (key_val, key.clone().asset_addr)
                                             }, "ASSETS", "ASSET_MANAGED_STATUS"),
        for_sale: MultiIndex::new(|t, key| (key.clone().for_sale, key.clone().asset_addr), "ASSETS", "ASSET_FOR_SALE"),
    };
    IndexedMap::new("ASSETS", indexes)
}




//  LEAGUES
pub struct LeagueIndexes<'a>{
    pub identifier: UniqueIndex<'a, Addr, LeagueInfo>,
    pub owner: MultiIndex<'a, Addr, LeagueInfo, Addr>,

}

impl<'a> IndexList<LeagueInfo> for LeagueIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<LeagueInfo>> + '_> {
        let v: Vec<&dyn Index<LeagueInfo>> = vec![&self.identifier, &self.owner];
        Box::new(v.into_iter())
    }
}

pub fn leagues<'a>()-> IndexedMap<'a, &'a  Addr, LeagueInfo, LeagueIndexes<'a>> {

    let indexes = LeagueIndexes {
        identifier: UniqueIndex::new(|d| d.clone().address, "LEAGUE"),
        owner: MultiIndex::new(|t, key| key.clone().owner, "LEAGUE", "LEAGUE_OWNER"),

    };
    IndexedMap::new("LEAGUE", indexes)
}




//   SEASONS
pub struct SeasonIndexes<'a>{
    pub identifier: UniqueIndex<'a, SeasonId, Season>,
    pub owning_league: MultiIndex<'a, LeagueAddr, Season, SeasonId>,
    pub season_start_date:  MultiIndex<'a, StartDate, Season , SeasonId>,
    pub season_start_and_end_date_by_league:  MultiIndex<'a, (LeagueAddr, StartDate, EndDate), Season , SeasonId>,

    pub season_end_date:  MultiIndex<'a, EndDate, Season , SeasonId>,
}

impl<'a> IndexList<Season> for SeasonIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<Season>> + '_> {
        let v: Vec<&dyn Index<Season>> = vec![&self.identifier, &self.owning_league,
                                              &self.season_start_date, &self.season_end_date,
                                              &self.season_start_and_end_date_by_league];
        Box::new(v.into_iter())
    }
}

pub fn seasons<'a>()-> IndexedMap<'a, SeasonId, Season, SeasonIndexes<'a>> {
    let indexes = SeasonIndexes {
        identifier: UniqueIndex::new(|d| (d.clone().id), "SEASON"),
        owning_league: MultiIndex::new(|t, key| key.clone().league, "SEASON", "SEASON_OWNER"),

        season_start_date: MultiIndex::new(|t, key|
                                               key.clone().start_date.seconds(),
                                           "SEASON", "SEASON_START_DATE"),
        season_start_and_end_date_by_league:
                        MultiIndex::new(|t, key|
                                                                 (key.clone().league, key.clone().start_date.seconds(),
                                                                  key.clone().end_date.seconds()) ,
                                        "SEASON", "SEASON_START_DATE"),
        season_end_date:
                        MultiIndex::new(|t, key| key.clone().end_date.seconds(),
                                        "SEASON", "SEASON_end_DATE"),
    };
    IndexedMap::new("SEASON", indexes)
}




// SEASON DEPOSITS



pub struct SeasonDepositsLedgerIndexes<'a>{
    pub identifier: UniqueIndex<'a, SeasonDepositId, SeasonLedger>,
    pub league: MultiIndex<'a, LeagueAddr, SeasonLedger, SeasonDepositId>,
    pub team:  MultiIndex<'a, TeamAddr, SeasonLedger , SeasonDepositId>,
    pub season: MultiIndex<'a, SeasonId, SeasonLedger , SeasonDepositId>,
    pub deposit: MultiIndex<'a, (String, u128), SeasonLedger , SeasonDepositId>,
    pub deposit_date: MultiIndex<'a, u64, SeasonLedger , SeasonDepositId>,
    pub withdrawal_date:  MultiIndex<'a, u64, SeasonLedger , SeasonDepositId>
}

impl<'a> IndexList<SeasonLedger> for SeasonDepositsLedgerIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<SeasonLedger>> + '_> {
        let v: Vec<&dyn Index<SeasonLedger>> = vec![&self.identifier, &self.league, &self.team,
                                              &self.season, &self.deposit, &self.deposit_date,
                                              &self.withdrawal_date];
        Box::new(v.into_iter())
    }
}


pub fn season_deposits_ledger<'a>() -> IndexedMap<'a, SeasonDepositId, SeasonLedger, SeasonDepositsLedgerIndexes<'a>> {
    let indexes = SeasonDepositsLedgerIndexes {
        identifier: UniqueIndex::new(|d| (d.clone().id), "SEASON_LEDGER"),
        league: MultiIndex::new(|t, key| key.clone().league, "SEASON_LEDGER", "SEASON_LEDGER_LEAGUE"),
        team: MultiIndex::new(|t, key| key.clone().team, "SEASON_LEDGER", "SEASON_LEDGER_TEAM"),
        season: MultiIndex::new(|t, key| key.clone().season_id, "SEASON_LEDGER", "SEASON_LEDGER_SEASON"),
        deposit: MultiIndex::new(|t, key|   (key.clone().team_deposit_amount.denom,  key.clone().team_deposit_amount.amount.u128()),
                                 "SEASON_LEDGER", "SEASON_LEDGER_DEPOSIT"),
        deposit_date:  MultiIndex::new(|t, key| key.clone().deposit_date.seconds(), "SEASON_LEDGER", "SEASON_LEDGER_DEPOSIT_DATE"),

        withdrawal_date:  MultiIndex::new(|t, key| {
            match key.clone().withdrawal_distribution_date {
                None => {
                   0u64
                }
                Some(d) => {
                    d.seconds()
                }
            }
        }, "SEASON_LEDGER", "SEASON_LEDGER_WITHDRAWAL_DATE"),
    };
    IndexedMap::new("SEASON_LEDGER", indexes)
}




//  TEAMS
pub struct TeamIndexes<'a>{
    pub identifier: UniqueIndex<'a, Addr, TeamInfo>,
    pub owner: MultiIndex<'a, OwnerAddr, TeamInfo, TeamAddr>,
    pub leagues:  MultiIndex<'a, LeagueAddr, TeamInfo, TeamAddr>,
}

impl<'a> IndexList<TeamInfo> for TeamIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<TeamInfo>> + '_> {
        let v: Vec<&dyn Index<TeamInfo>> = vec![&self.identifier, &self.owner, &self.leagues];
        Box::new(v.into_iter())
    }
}


pub fn teams<'a>()-> IndexedMap<'a, TeamAddr, TeamInfo, TeamIndexes<'a>> {
    let indexes = TeamIndexes {
        identifier: UniqueIndex::new(|d| d.clone().address, "TEAM"),
        owner: MultiIndex::new(|t, key| key.clone().owner, "TEAM", "TEAM_OWNER"),
        //players: MultiIndex::new(|t, key| (key.clone().player.address, key.clone().address ), "TEAM", "TEAM_PLAYER"),
        leagues: MultiIndex::new(|t, key|
                                     {
                                         match key.clone().league_assigned {
                                             Some(l) => l.league,
                                             None => Addr::unchecked("[-|---0]")
                                         }
                                     } , "TEAM", "TEAM_LEAGUE"),
    };
    IndexedMap::new("TEAM", indexes)
}


//  INVITATIONS

pub struct InvitesIndexes<'a>{
    pub identifier: UniqueIndex<'a, MessageId, Message<JoinSeasonRequestInfo>>,
    pub sender: MultiIndex<'a, (AsseTypes_u8, Addr), Message<JoinSeasonRequestInfo>, MessageId>,
    pub recipient: MultiIndex<'a, (AsseTypes_u8, Addr), Message<JoinSeasonRequestInfo>, MessageId>,
    pub season_id: MultiIndex<'a, SeasonId, Message<JoinSeasonRequestInfo>, MessageId>,
    pub season_from_to: MultiIndex<'a, (SeasonId, TeamAddr, LeagueAddr), Message<JoinSeasonRequestInfo>, MessageId>,
    pub from_to:  MultiIndex<'a, (Addr, Addr), Message<JoinSeasonRequestInfo>, MessageId>,
}



impl<'a> IndexList<Message<JoinSeasonRequestInfo>> for InvitesIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<Message<JoinSeasonRequestInfo>>> + '_> {
        let v: Vec<&dyn Index<Message<JoinSeasonRequestInfo>>> = vec![&self.identifier, &self.sender,
                                                                      &self.recipient, &self.season_id,
                                                                      &self.season_from_to];
        Box::new(v.into_iter())
    }
}

pub fn join_season_requests<'a>() -> IndexedMap<'a, MessageId, Message<JoinSeasonRequestInfo>, InvitesIndexes<'a>> {
    let indexes = InvitesIndexes {
        identifier: UniqueIndex::new(|d| (d.clone().id), "INVITE"),
        sender: MultiIndex::new(|t, key|{
            (key.clone().delivery.from.asset_type.to_u8(), key.clone().delivery.from.address)
        }, "INVITE", "INVITE_SENDER"),
        recipient: MultiIndex::new(|t, key| {
            (key.clone().delivery.to.asset_type.to_u8(), key.clone().delivery.to.address)
        }, "INVITE", "INVITE_RECIPIENT"),
        season_id: MultiIndex::new(|t, key| key.clone().data.season_id,
                                   "INVITE", "INVITE_SEASON_ID"),

        season_from_to: MultiIndex::new(|t, key|
                                            (key.clone().data.season_id,
                                             key.clone().delivery.from.address,
                                             key.clone().delivery.to.address),
                                        "INVITE", "INVITE_SEASON_ID"),
        from_to: MultiIndex::new(|t, key|
                                     {
                                         (key.clone().delivery.from.address, key.clone().delivery.to.address)
                                     },
                                 "INVITE", "INVITE_FROM_TO"),
    };
    IndexedMap::new("INVITE", indexes)
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub native_denom: String,
}

// This stores the config variables during initialization of the contract
//pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");


pub const MANAGEMENT: ManagementService = ManagementService::new("management_item");
pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("cw4-hooks");
pub const TOTAL: Item<u64> = Item::new(TOTAL_KEY);

pub const MEMBERS: SnapshotMap<&Addr, u64> = SnapshotMap::new(
    cw4::MEMBERS_KEY,
    cw4::MEMBERS_CHECKPOINTS,
    cw4::MEMBERS_CHANGELOG,
    Strategy::EveryBlock,
);
