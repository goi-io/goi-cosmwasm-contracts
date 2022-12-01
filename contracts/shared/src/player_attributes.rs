use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SideOfBall {
    Offense,
    Defense,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AttrItem {
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PlayerAttributes {
    pub hands: AttrItem,
    pub accuracy: AttrItem,
    pub speed: AttrItem,
    pub strength: AttrItem,
    pub leader: AttrItem,
    pub pressure_threshold: AttrItem,
    pub agility: AttrItem,
    pub football_iq: AttrItem,
    pub temperament: AttrItem,
    pub angle_of_view: u8
}







#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Positions {

    //Offense
    RB ,
    QB ,
    WR1 ,
    WR2 ,
    CO ,
    GL ,
    GR ,

    //Defense
    S ,
    CB1 ,
    CB2 ,
    LB ,
    CD ,
    TR ,
    TL ,
}

impl Positions {
    pub fn from_string (item: &String) -> Option<Positions> {
        match item.to_lowercase().as_ref() {
            //Offense
            "rb" => Some( Positions::RB),
            "qb" => Some( Positions::QB),
            "wr1" => Some( Positions::WR1),
            "wr2" => Some( Positions::WR2),
            "co" => Some( Positions::CO),
            "gl" => Some( Positions::GL),
            "gr" => Some( Positions::GR),

            //Defense
            "s" => Some( Positions::S),
            "cb1" => Some( Positions::CB1),
            "cb2" => Some( Positions::CB2),
            "lb" => Some( Positions::LB),
            "cd" => Some( Positions::CD),
            "tr" => Some( Positions::TR),
            "tl" => Some( Positions::TL),
            _ => None,
        }
    }
    pub fn to_string(&mut self) -> String {
        match self {
            //Offense
            Positions::RB => "rb".to_string(),
            Positions::QB => "qb".to_string(),
            Positions::WR1 => "wr1".to_string(),
            Positions::WR2 => "wr2".to_string(),
            Positions::CO => "co".to_string(),
            Positions::GL => "gl".to_string(),
            Positions::GR => "gr".to_string(),

            //Defense
            Positions::S => "s".to_string(),
            Positions::CB1 => "cb1".to_string(),
            Positions::CB2 => "cb2".to_string(),
            Positions::LB => "lb".to_string(),
            Positions::CD => "cd".to_string(),
            Positions::TR => "tr".to_string(),
            Positions::TL => "tl".to_string(),
        }
    }
}








