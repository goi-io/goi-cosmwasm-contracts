use cosmwasm_std::{Addr, BlockInfo, Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::ModelItem;
use crate::league::SeasonActiveStatus;
use crate::messaging::DeliveryInfo;
use crate::utils::{EndDate, LeagueAddr, SeasonId, StartDate, TeamAddr, MAX_TEAMS_ALLOWED};
use crate::utils::general::merge_data;





#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SeasonAccessTypes {
    Open,
    WinnerTakeAll { coin: Coin }
}

impl Default for SeasonAccessTypes {
    fn default() -> Self {
        SeasonAccessTypes::Open
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SeasonStatus {
    Active,
    Private,
    Cancelled { date_cancelled: BlockInfo},
}

impl Default for SeasonStatus {
    fn default() -> Self {
        SeasonStatus::Private
    }
}





    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct SeasonModelData {
        pub description: ModelItem<Option<String>>,
        pub start_date: ModelItem<Option<Timestamp>>,
        pub end_date: ModelItem<Option<Timestamp>>,
        pub access_type: ModelItem<Option<SeasonAccessTypes>>,
        pub status: ModelItem<Option<SeasonStatus>>,
        pub max_teams_allowed: ModelItem<Option<u32>>,
    }


    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct SeasonLedger {
        pub id: u64,
        pub season_id: SeasonId,
        pub league: LeagueAddr,
        pub team: TeamAddr,
        pub team_deposit_amount: Coin,
        pub deposit_date: Timestamp,
        pub withdrawal_distribution_date: Option<Timestamp>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Season {
        pub id: SeasonId,
        pub league: Addr,
        pub name: String,
        pub current_episode: u32,
        pub description: Option<String>,
        pub start_date: Timestamp,
        pub end_date: Timestamp,
        pub access_type: Option<SeasonAccessTypes>,
        pub status: Option<SeasonStatus>,
        pub max_teams_allowed: Option<u32>,
    }

    impl Season {
        pub fn new(id: u64, owning_league: Addr, name: String, description: Option<String>, block: BlockInfo) -> Self {
            let res = 
                    Season {
                        id,
                        league: owning_league,
                        name,
                        current_episode: 0,
                        description,
                        start_date: Default::default(),
                        end_date: Default::default(),
                        access_type: Default::default(),
                        status: Default::default(),
                        max_teams_allowed: Some(MAX_TEAMS_ALLOWED),
                    };
            if !res.validate(block){
                panic!("Invalid season!")
            }
            else {
                res
            }
                    
        }

        pub fn validate(&self, block: BlockInfo) -> bool{
            let mut error_count:u32 = 0;

            //updates are nolonger possible after season starts
            match block.clone().time.seconds() >  self.start_date.seconds(){
                true => {
                    error_count = error_count + 1;
                }
                false => {
                    //Check to see if season has started,
                    //if so, we don't allow any more updates
                    match self.start_date < block.time {
                        true =>  error_count  = error_count + 1,
                        false => ()
                    }
                    match self.start_date >= self.end_date {
                        true =>  error_count  = error_count + 1,
                        _ => ()
                    }

                    match self.name.is_empty() {
                        true =>  error_count  = error_count + 1,
                        false => ()
                    }


                    match self.max_teams_allowed == Some(MAX_TEAMS_ALLOWED) {
                        true =>  (),
                        false => error_count  = error_count + 1,
                    }
                }
            }

            error_count == 0
        }

        pub fn merge_updates(&self, model: SeasonModelData, block: BlockInfo) -> Self
        {
            let res = 
                    Season {
                        
                        league: self.league.clone(),
                        name: self.name.clone(),
                        id: self.id,
                        current_episode: self.current_episode,

                        description:
                        merge_data(self.description.clone(),
                                model.description),
                        start_date: {
                            match model.start_date.data {
                                None => {
                                    self.start_date.clone()
                                },
                                Some(sd) => {
                                sd
                                }
                            }

                        },
                        end_date: {
                            match model.end_date.data {
                                None => {
                                    self.end_date.clone()
                                }
                                Some(ed) => {
                                    ed
                                }
                            }

                        } ,
                        access_type:
                        merge_data(self.access_type.clone(),
                                model.access_type),
                        status:
                        merge_data(self.status.clone(),
                                model.status),
                        max_teams_allowed: Some(MAX_TEAMS_ALLOWED),

                        
                    };
            if !res.validate(block){
                panic!("Invalid season!")
            }
            else {
                res
            }
            
        }
    }


