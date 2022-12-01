pub use crate::error::GoiError;

mod error;


#[path = "./domain/goi_manager.rs"]
pub mod goi_manager;


#[path = "./domain/player.rs"]
pub mod player;

#[path = "./domain/league.rs"]
pub mod league;

#[path = "./domain/season.rs"]
pub mod season;

#[path = "./domain/data.rs"]
pub mod data;

#[path = "./utils/utils.rs"]
pub mod utils;
pub mod player_attributes;

#[path = "./domain/rewards.rs"]
pub mod rewards;

#[path = "./domain/team.rs"]
pub mod team;


#[path = "domain/application.rs"]
pub mod application;

#[path = "./domain/task.rs"]
pub mod task;

#[path = "./domain/query_response_info.rs"]
pub mod query_response_info;


#[path = "./domain/manage.rs"]
pub mod manage;


#[path = "./domain/saleable.rs"]
pub mod saleable;

#[path = "./domain/display.rs"]
pub mod display;

#[path = "./domain/link_team_player.rs"]
pub mod link_team_player;


#[path = "./domain/messaging.rs"]
pub mod messaging;



