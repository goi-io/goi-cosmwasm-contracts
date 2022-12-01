use std::collections::HashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::league::LeagueTeamAssignment;

use crate::manage::ManagedStatus;
use crate::player::PlayerInfo;
use crate::utils::{BlockTime, IManaged, LeagueAddr, OwnerAddr, PlayerAddr, TeamAddr};
use crate::utils::general::AssetTypes;





#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TeamInfo {
    pub address: TeamAddr,
    pub league_assigned: Option<LeagueTeamAssignment>,
    pub name: String,
    pub created: BlockTime,
    pub owner: OwnerAddr,


}

impl TeamInfo {
    // Constructs a new instance of [`Second`].
    // Note this is an associated function - no self.
    pub fn new(league_assigned: Option<LeagueTeamAssignment>, team_addr: TeamAddr, name: String, player: PlayerInfo, owner: OwnerAddr, block_time: BlockTime) -> Self {
        Self {

            address: team_addr,
            league_assigned,
            name,
            owner,
            created: BlockTime
                {
                    height: block_time.height,
                    time: block_time.time,
                    chain_id: block_time.chain_id
                },


        }
    }
}