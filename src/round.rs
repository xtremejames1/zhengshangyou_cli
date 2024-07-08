use crate::play;
use crate::player;

pub struct Round {
    pub plays: Vec<play::Play>,         // List of plays in the round
    pub winner: Option<player::Player>, // Player who wins the round
}

impl Round {
    pub fn new() -> Self {
        let plays = Vec::new();
        let winner = None;
        Self { plays, winner }
    }

    pub fn add_play(&mut self, play: play::Play) {
        // check if play is valid?
        self.plays.push(play);
    }

    pub fn end_round(&mut self) {
        self.winner = Some(self.plays.last().unwrap().player.clone());
    }
}
