use crate::card;
use crate::player;

pub struct Play {
    pub class: Class,
    pub rank: card::Rank,
    pub cards: Vec<card::Card>,
    pub player: player::Player,
}

impl Play {
    pub fn new(player: player::Player) -> Self {
        let cards = Vec::new();
        let rank = card::Rank::Three; // set the rank to the lowest possible
        let class = Class::Invalid;
        Self {
            class,
            rank,
            cards,
            player,
        }
    }

    pub fn set_cards(&mut self, cards: Vec<card::Card>) {
        self.cards = cards;
        self.identify_play();
    }

    pub fn add_card(&mut self, c: card::Card) {
        self.cards.push(c);
        self.identify_play();
    }

    fn identify_play(&mut self) {
        self.class = identify_class(&mut self.cards);
        self.rank = identify_rank(&mut self.cards);
    }
}

pub fn identify_class(cards: &mut Vec<card::Card>) -> Class {
    match cards.len() {
        1 => Class::Single,
        2 => {
            if homogenous(cards.to_vec()) {
                Class::Double
            } else {
                Class::Invalid
            }
        }
        3 => {
            if homogenous(cards.to_vec()) {
                Class::Triple
            } else {
                Class::Invalid
            }
        }
        4 => {
            if homogenous(cards.to_vec()) {
                Class::Quad
            } else {
                Class::Invalid
            }
        }
        _ => straight(cards),
    }
}

pub fn identify_rank(cards: &mut Vec<card::Card>) -> card::Rank {
    // Sort and get the highest ranked card
    cards.sort_unstable();
    cards.last().unwrap().rank
}

fn homogenous(cards: Vec<card::Card>) -> bool {
    if cards.is_empty() {
        return false;
    } // cannot be homogenous if no cards
    for card in &cards {
        if *card != cards[0] {
            return false;
        }
    }
    true
}

fn straight(cards: &mut Vec<card::Card>) -> Class {
    if cards.len() < 5 {
        return Class::Invalid;
    } // cannot be straight if not enough cards
    cards.sort_unstable(); // sort cards

    if cards[2].rank as usize == cards[1].rank as usize
        && cards[1].rank as usize == cards[2].rank as usize
    {
        // check for triple straight
        if cards.len() % 3 != 0 {
            return Class::Invalid;
        }

        for i in 2..cards.len() / 3 {
            if cards[i * 3 - 1].rank as usize != cards[i * 3 - 2].rank as usize
                || cards[i * 3 - 2].rank as usize != cards[i * 3 - 3].rank as usize
                || cards[i * 3 - 3].rank as usize - 1 != cards[i * 3 - 4].rank as usize
            {
                return Class::Invalid;
            }
        }
        return Class::TripleStraight;
    }

    if cards[1].rank as usize == cards[0].rank as usize {
        // check for double straight
        // make sure length is even
        if cards.len() % 2 != 0 {
            return Class::Invalid;
        }

        for i in 2..cards.len() / 2 {
            if cards[i * 2 - 1].rank as usize != cards[i * 2 - 2].rank as usize
                || cards[i * 2 - 2].rank as usize - 1usize != cards[i * 2 - 3].rank as usize
            {
                return Class::Invalid;
            }
        }
        return Class::DoubleStraight;
    }

    if cards[1].rank as usize - 1 == cards[0].rank as usize {
        // check for single straight
        for i in 2..cards.len() {
            if cards[i].rank as usize - 1 != cards[i - 1].rank as usize {
                return Class::Invalid;
            }
        }
        return Class::SingleStraight;
    }

    Class::Invalid
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Class {
    Invalid,
    Single,
    Double,
    Triple,
    Quad,
    SingleStraight,
    DoubleStraight,
    TripleStraight,
}
