use crate::card;
use crate::player;

pub struct Play {
    pub class: Class,
    pub rank: card::Rank,
    pub cards: Vec<card::Card>,
    pub player: player::Player,
}

impl Play {
    pub fn new(p: player::Player) -> Self {
        let mut cards = Vec::new();
        let mut rank = card::Rank::Three; // set the rank to the lowest possible
        let mut class = Class::Invalid;
        let player = p;
        Self {
            class,
            rank,
            cards,
            player
        }
    }

    pub fn add_card(&mut self, c: card::Card) {
        self.cards.push(c);
    }

    pub fn identify_class(&mut self) {
        self.class = match self.cards.len() {
            1 => Class::Single,
            2 => if self.homogenous() {Class::Double} else {Class::Invalid},
            3 => if self.homogenous() {Class::Triple} else {Class::Invalid},
            4 => if self.homogenous() {Class::Quad} else {Class::Invalid},
            _ => Class::Invalid
        };

        // Sort and get the highest ranked card
        self.cards.sort_unstable();
        self.rank = self.cards.last().unwrap().rank;
    }

    fn homogenous (&mut self) -> bool {
        if self.cards.is_empty() { return false; } // cannot be homogenous if no cards
        for card in &self.cards {
            if *card != self.cards[0] {
                false
            }
        }
        true
    }

    fn straight (&mut self) -> Class {
        if self.cards.len >= 5 { return Class::Invalid; } // cannot be straight if not enough cards 
        self.cards.sort_unstable();
        if self.cards[1] - 1 == self.cards[i - 1]; { // check for single straight
            for i 2..self.cards.len() {
                if self.cards[i] - 1 != self.cards[i - 1] {
                    Class::Invalid
                }
            }
            Class::SingleStraight
        }
        if self.cards[1] == self.cards[i - 1]; { // check for double straight
            // make sure length is even
            if(self.cards.len() % 2 != 0) { Class::Invalid }

            for i 2..self.cards.len()/2 {
                if self.cards[i*2] == self.cards[i*2 - 1] && self.cards[i*2] - 1 == self.cards[i*2 - 2] {
                    Class::Invalid
                }
            }
            Class::DoubleStraight
        }
    }
}

pub enum Class {
    Invalid,
    Single,
    Double,
    Triple,
    Quad,
    SingleStraight,
    DoubleStraight,
    TripleStraight
}
