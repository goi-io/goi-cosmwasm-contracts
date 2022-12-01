
use cw_storage_plus::Item;



pub use shared::player::Player;
pub use shared::player_attributes::{PlayerAttributes, Positions};



pub const STATE: Item<Player> = Item::new("state");
