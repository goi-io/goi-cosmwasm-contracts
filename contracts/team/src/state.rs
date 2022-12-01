use cosmwasm_std::Addr;
use cw4::TOTAL_KEY;
use cw_controllers::{Admin, Hooks};
use cw_storage_plus::{Item, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use managed::service::ManagedService;
use saleable::service::SaleableService;

use crate::team_attributes::TeamPlayers;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
    pub players: TeamPlayers,
}


pub const STATE: Item<State> = Item::new("state");

pub const SALEABLE_SERVICE: SaleableService = SaleableService::new("saleable_service");
pub const MANAGEABLE_SERVICE: ManagedService = ManagedService::new("manageable_service");
pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("cw4-hooks");
pub const TOTAL: Item<u64> = Item::new(TOTAL_KEY);

pub const MEMBERS: SnapshotMap<&Addr, u64> = SnapshotMap::new(
    cw4::MEMBERS_KEY,
    cw4::MEMBERS_CHECKPOINTS,
    cw4::MEMBERS_CHANGELOG,
    Strategy::EveryBlock,
);
