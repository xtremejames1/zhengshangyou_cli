use crate::card;

#[derive(Clone, PartialEq)]
pub struct Hand {
    pub cards: Vec<card::Card>,
}

impl Hand {
    pub fn add_card(&mut self, c: card::Card) {
        self.cards.push(c);
    }

    pub fn new() -> Self {
        let mut cards = Vec::new();
        Self { cards }
    }

    pub fn sort(&mut self) {
        self.cards.sort_unstable();
    }
}
