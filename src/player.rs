use crate::hand;

#[derive(Clone)]
pub struct Player {
    pub hand: hand::Hand,
    pub name: String,
    pub score: u16,
}

impl Player {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let hand = hand::Hand::new();
        let score = 0u16;
        Self {
            hand,
            name: name.into(),
            score,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl AsRef<Player> for Player {
    fn as_ref(&self) -> &Player {
        self
    }
}
