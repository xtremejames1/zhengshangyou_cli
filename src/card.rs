use std::cmp::Ordering;
use std::fmt;
use strum_macros::EnumIter;

#[derive(Clone, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if &self.rank == &Rank::Joker {
            write!(f, "{:?} Joker", &self.suit)
        } else {
            let rank_str = match &self.rank {
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "10",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
                Rank::Two => "2",
                Rank::Joker => "Joker",
            };
            let suit_str = match &self.suit {
                Suit::Spades => "♠",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
                Suit::Hearts => "♥",
                Suit::Red | Suit::Black => "",
            };
            write!(f, "{}{}", rank_str, suit_str)
        }
    }
}

impl Into<String> for Card {
    fn into(self) -> String {
        if self.rank == Rank::Joker {
            let suit_str = match &self.suit {
                Suit::Red => "R",
                Suit::Black => "B",
                _ => "",
            };
            return format!("{suit_str}O");
        } else {
            let rank_str = match self.rank {
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "0",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
                Rank::Two => "2",
                Rank::Joker => "O",
            };
            let suit_str = match &self.suit {
                Suit::Spades => "♠",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
                Suit::Hearts => "♥",
                _ => "",
            };
            return format!("{}{}", rank_str, suit_str);
        }
    }
}

impl From<String> for Card {
    fn from(value: String) -> Self {
        let card_str = value.split_at(1);
        let rank = match card_str.0 {
            "3" => Rank::Three,
            "4" => Rank::Four,
            "5" => Rank::Five,
            "6" => Rank::Six,
            "7" => Rank::Seven,
            "8" => Rank::Eight,
            "9" => Rank::Nine,
            "0" => Rank::Ten,
            "J" => Rank::Jack,
            "Q" => Rank::Queen,
            "K" => Rank::King,
            "A" => Rank::Ace,
            "2" => Rank::Two,
            "O" => Rank::Joker,
            _ => Rank::Three,
        };
        let suit = match card_str.1 {
            "♠" => Suit::Spades,
            "♦" => Suit::Diamonds,
            "♣" => Suit::Clubs,
            "♥" => Suit::Hearts,
            "R" => Suit::Red,
            "B" => Suit::Black,
            _ => Suit::Red,
        };

        Card::new(suit, rank)
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.rank as usize).cmp(&(other.rank as usize))
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}

#[derive(EnumIter, Debug, Clone, PartialEq, Eq, Copy)]
pub enum Suit {
    Spades,
    Diamonds,
    Clubs,
    Hearts,
    Red,
    Black,
}

#[derive(EnumIter, Debug, Clone, PartialEq, Eq, Copy)]
pub enum Rank {
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
    Two = 15,
    Joker = 16,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
