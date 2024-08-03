use std::net::TcpStream;

use crate::hand;

#[derive(Clone)]
pub struct Player {
    pub hand: hand::Hand,
    pub name: String,
    pub score: u16,
}

impl Player {
    pub fn new(name: String) -> Self {
        let hand = hand::Hand::new();
        let score = 0u16;
        Self { hand, name, score }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
