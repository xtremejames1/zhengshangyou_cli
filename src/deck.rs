use crate::card;
use strum::IntoEnumIterator;

pub struct Deck {
    pub cards: Vec<card::Card>,
}

impl Deck {
    pub fn add_card(&mut self, c: card::Card) {
        self.cards.push(c);
    }

    pub fn new(n: u32) -> Self {
        let mut cards = Vec::new();
        for _ in 0u32..n {
            // Iterate through possible cards, minus Jokers
            for suit in card::Suit::iter() {
                if suit != card::Suit::Red && suit != card::Suit::Black {
                    for rank in card::Rank::iter() {
                        if rank != card::Rank::Joker {
                            cards.push(card::Card::new(suit.clone(), rank.clone()))
                        }
                    }
                }
            }
            // Add Jokers
            cards.push(card::Card::new(card::Suit::Red.clone(), card::Rank::Joker.clone()));
            cards.push(card::Card::new(card::Suit::Black.clone(), card::Rank::Joker.clone()));
        }
        Self { cards }
    }

    pub fn draw_card(&mut self) -> card::Card {
        let index = (rand::random::<f32>() * self.cards.len() as f32).floor() as usize;
        self.cards.remove( index )
    }

    pub fn is_empty(&mut self) -> bool {
        self.cards.is_empty()
    }

    pub fn sort(&mut self) {
        self.cards.sort_unstable();
    }
}
