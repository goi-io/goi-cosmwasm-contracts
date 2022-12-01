use crate::utils::{PlayerAddr, TeamAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LinkTeamPlayer {
    pub player_address: PlayerAddr,
    pub team_address: TeamAddr,

}