use crate::hand;

#[derive(Clone)]
pub struct PlayerClient {
    pub num_cards: u16,
    pub name: String,
    pub score: u16,
}

impl PlayerClient {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            num_cards: 0u16,
            name: name.into(),
            score: 0u16,
        }
    }
}

impl PartialEq for PlayerClient {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl AsRef<PlayerClient> for PlayerClient {
    fn as_ref(&self) -> &PlayerClient {
        self
    }
}
