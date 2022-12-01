use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shared::player::PlayerInfo;
use shared::player_attributes::{Positions, SideOfBall};

use crate::TeamError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TeamPosition {
    player: Option<Addr>,
    position: Positions,
    side_of_ball: SideOfBall
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TeamPlayers
{
    //Offense
    pub rb: Option<Addr>,
    pub qb: Option<Addr>,
    pub wr1: Option<Addr>,
    pub wr2: Option<Addr>,
    pub co: Option<Addr>,
    pub gl: Option<Addr>,
    pub gr: Option<Addr>,

    //Defense
    pub s: Option<Addr>,
    pub cb1: Option<Addr>,
    pub cb2: Option<Addr>,
    pub lb: Option<Addr>,
    pub cd: Option<Addr>,
    pub tr: Option<Addr>,
    pub tl: Option<Addr>,
}

impl TeamPlayers {
    pub fn get_player_at_position(&mut self, pos: Positions ) -> Option<Addr>{
        match pos {
            Positions::RB => self.rb.clone(),
            Positions::QB  => self.qb.clone(),
            Positions::WR1 => self.wr1.clone(),
            Positions::WR2  => self.wr2.clone(),
            Positions::CO => self.co.clone(),
            Positions::GL => self.gl.clone(),
            Positions::GR => self.gr.clone(),
            Positions::S => self.s.clone(),
            Positions::CB1 => self.cb1.clone(),
            Positions::CB2 => self.cb2.clone(),
            Positions::LB => self.lb.clone(),
            Positions::CD => self.cd.clone(),
            Positions::TR => self.tr.clone(),
            Positions::TL => self.tl.clone(),
        }
    }

    pub fn all_players (&mut self) -> Vec<TeamPosition> {
           vec![ TeamPosition{ player: self.rb.clone(), position: Positions::RB, side_of_ball: SideOfBall::Offense },
            TeamPosition{ player: self.qb.clone(), position: Positions::QB, side_of_ball: SideOfBall::Offense },
            TeamPosition{ player: self.wr1.clone(), position: Positions::WR1, side_of_ball: SideOfBall::Offense },
            TeamPosition{ player: self.wr2.clone(), position: Positions::WR2, side_of_ball: SideOfBall::Offense },
            TeamPosition{ player: self.co.clone(), position: Positions::CO , side_of_ball: SideOfBall::Offense},
            TeamPosition{ player: self.gl.clone(), position: Positions::GL, side_of_ball: SideOfBall::Offense },
            TeamPosition{ player: self.gr.clone(), position: Positions::GR, side_of_ball: SideOfBall::Offense },

            TeamPosition{ player: self.s.clone(), position: Positions::S, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.cb1.clone(), position: Positions::CB1, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.cb2.clone(), position: Positions::CB2, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.lb.clone(), position: Positions::LB, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.cd.clone(), position: Positions::CD, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.tr.clone(), position: Positions::TR, side_of_ball: SideOfBall::Defense },
            TeamPosition{ player: self.tl.clone(), position: Positions::TL, side_of_ball: SideOfBall::Defense }
        ]
    }

    pub fn get_player(&mut self, player: Addr) -> Option<TeamPosition> {
        let item =
            self.all_players().into_iter()
                .filter(|i|  match i.player {Some (_) => true, _ => false} )
                .find(|i| match &i.player {
                    Some (a) => *a == player,
                    _ => false
                }  );
        item
    }


    pub fn get_player_by_position (&mut self, pos: Positions) -> Vec<TeamPosition> {
         self.all_players().into_iter()
             .filter(|i| i.position == pos )
             .collect()
    }

    pub fn get_players_by_side_of_players (&mut self, sob: SideOfBall) -> Vec<TeamPosition> {
        self.all_players().into_iter()
            .filter(|i| i.side_of_ball == sob )
            .collect()
    }


    pub fn get_positions (&mut self, filled_status: bool) -> Vec<TeamPosition> {
        self.all_players().into_iter()
            .filter(|i| match filled_status { true => i.player.is_some(), false => i.player.is_none() }  )
            .collect()
    }
    fn unset_position(&mut self, position: Positions) -> Result<(), TeamError> {
        match position {
            Positions::RB => self.rb = None,
            Positions::QB  => self.qb = None,
            Positions::WR1 => self.wr1 = None,
            Positions::WR2  => self.wr2 = None,
            Positions::CO => self.co = None,
            Positions::GL => self.gl = None,
            Positions::GR => self.gr = None,
            Positions::S => self.s = None,
            Positions::CB1 => self.cb1 = None,
            Positions::CB2 => self.cb2 = None,
            Positions::LB => self.lb = None,
            Positions::CD => self.cd = None,
            Positions::TR => self.tr = None,
            Positions::TL => self.tl = None,
        }
        Ok(())
    }

    fn set_position(&mut self, addr: Addr, position: Positions) -> Result<(), TeamError> {
        match position {
            Positions::RB => self.rb = Some (addr),
            Positions::QB  => self.qb = Some (addr),
            Positions::WR1 => self.wr1 = Some (addr),
            Positions::WR2  => self.wr2 = Some (addr),
            Positions::CO => self.co = Some (addr),
            Positions::GL => self.gl = Some (addr),
            Positions::GR => self.gr = Some (addr),
            Positions::S => self.s = Some (addr),
            Positions::CB1 => self.cb1 = Some (addr),
            Positions::CB2 => self.cb2 = Some (addr),
            Positions::LB => self.lb = Some (addr),
            Positions::CD => self.cd = Some (addr),
            Positions::TR => self.tr = Some (addr),
            Positions::TL => self.tl = Some (addr),
        }
        Ok(())
    }
    pub fn add_player_to_team(&mut self, addr: Addr, pos: Positions) -> Result<(), TeamError> {
        let res = self.get_player_at_position(pos.clone());
        match res {
            None => self.set_position(addr, pos.clone()),
            Some(_) => Err(TeamError::PositionAlreadyAssigned { })
        }
    }


    pub fn remove_players_from_positions(&mut self, players: Vec<PlayerInfo>) -> Result<(), TeamError> {
        let mut res: Result<(), TeamError>  = Ok(());
        for item in players {
            let player_addr = self.get_player_at_position(item.position.clone());
            match player_addr {
                Some(p_addr) =>  match p_addr == item.address  {
                    true => res = self.unset_position(item.position.clone()),
                    false => res = Err(TeamError::RequestedPlayerPositionConflict
                    { player_address: item.address.clone(), position: item.position.clone() })
                },
                None =>  res = Err(TeamError::PositionNotAssigned { position: item.position })
            };
            match res {
                Ok(_) => continue,
                Err(_) => break,
            }
        };
        res

    }

}
