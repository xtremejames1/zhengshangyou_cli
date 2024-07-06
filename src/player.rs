use crate::hand;

pub struct Player {
    pub hand: hand::Hand,
    pub name: String,
    pub score: u16,
}

impl Player {
    pub fn new(name: String) -> Self {
        let mut hand = hand::Hand::new();
        let score = 0u16;
        Self {
            hand,
            name,
            score
        }
    }
}
