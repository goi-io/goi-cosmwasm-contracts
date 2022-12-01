use cosmwasm_std::{Addr, StdError, Timestamp, BlockInfo};
use thiserror::Error;

use group_admin::GroupAdminError;
use manager::error::ManagementError;
use shared::player::PlayerInfo;
use shared::utils::{MessageId, LeagueAddr, SeasonId, TeamAddr};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),


    #[error("{0}")]
    GroupAdminHooksError(#[from] GroupAdminError),

    #[error("ManagementExecError")]
    ManagementExecError {management_error: ManagementError},

    #[error("SingletonContractAlreadyInitialized")]
    SingletonContractAlreadyInitialized {},

    #[error("Unauthorized")]
    Unauthorized { sender: Addr},

    #[error("PlayerNameAlreadyInUse")]
    PlayerNameAlreadyInUse { first_name: String, last_name: String},

    #[error("AddPlayerErrors")]
    AddPlayerErrors { players_assigned_to_another_team: Vec<PlayerInfo>,
                      source_dupe_name_count: i32,
                      unauthorized_request: bool},


    #[error("DuplicateNamesFoundInSource")]
    DuplicateNamesFoundInSource { },


    #[error("InvalidSeason")]
    InvalidSeason { },


    #[error("InvalidSeasonJoinRequest")]
    InvalidSeasonJoinRequest { },


    #[error("InvalidTeamSubmissions")]
    InvalidTeamSubmissions{ },

    #[error("InvalidTeamSubmissions")]
    TeamAlreadyMemberOfALeague{ team_addr: TeamAddr, league_assigned_to: LeagueAddr },



    #[error("TeamAlreadyMemberOfSeason")]
    TeamAlreadyMemberOfSeason{ },



    #[error("SeasonScheduleConflict")]
    SeasonScheduleConflict {  conflicting_season: Vec<SeasonId> },


    #[error("SeasonTypeNotSet")]
    SeasonTypeNotSet { },


    #[error("TooLateToSendInvitesForSeason")]
    TooLateToSendInvitesForSeason {  },


    #[error("TooLateToRequestToJoinLeagueSeason")]
    TooLateToRequestToJoinLeagueSeason {  },



    #[error("TooLateToCancelSeason")]
    TooLateToCancelSeason {  },



    #[error("ItemNotFound")]
    ItemNotFound {  item_address: Addr},


    #[error("IncorrectFundingSent")]
    IncorrectFundingSent { },


    #[error("SeasonNotFound")]
    SeasonNotFound { },


    #[error("TeamNotMemberOfSeason")]
    TeamNotMemberOfSeason { },

    #[error("SeasonDepositAlreadyClaimed")]
    SeasonDepositAlreadyClaimed { },

    #[error("SeasonStatusCancelled")]
    SeasonStatusCancelled { date_cancelled: BlockInfo },



    #[error("SeasonHasReachedCapacity")]
    SeasonHasReachedCapacity { },


    #[error("SeasonStatusNotSet")]
    SeasonStatusNotSet { },


    #[error("SeasonStatusPrivate")]
    SeasonStatusPrivate { },


    #[error("InviteNotFound")]
    InviteNotFound {  invite_id: MessageId },


    #[error("InvitationNotFoundOrIncorrectSeason")]
    InvitationNotFoundOrIncorrectSeason {  invite_message_id: MessageId, leave_league_at_end_of_season_id: SeasonId },


    #[error("ErrorProcessingRequest")]
    ErrorProcessingRequest {  request: String},


}


impl From<ManagementError> for ContractError {
    fn from(err: ManagementError) -> Self {
        ContractError::ManagementExecError{ management_error: err }
    }

}

