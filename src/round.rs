use crate::play;

pub struct Round {
    pub plays: Vec<play::Play>, // List of plays in the round
}

impl Round {
    pub fn new() -> Self {
        let plays = Vec::new();
        Self { plays }
    }

    pub fn add_play(&mut self, play: play::Play) {
        self.plays.push(play);
    }
}

impl AsMut<Round> for Round {
    fn as_mut(&mut self) -> &mut Round {
        self
    }
}
