use crate::card;
use crate::deck;
use crate::display;
use crate::play;
use crate::player;
use crate::round;

pub struct Game {
    pub players: Vec<player::Player>,
    pub rounds: Vec<round::Round>,
    pub deck: deck::Deck,
}

impl Game {
    pub fn new(players: Vec<player::Player>, deck: deck::Deck) -> Self {
        let rounds = Vec::new();
        display::init();
        Self {
            players,
            rounds,
            deck,
        }
    }

    pub fn start_game(&mut self) {
        self.deal_cards();
        self.player_moves();
    }

    pub fn deal_cards(&mut self) {
        // pick random player to start, give them a 3 of hearts as standard in the game
        let index = (rand::random::<f32>() * self.players.len() as f32).floor() as usize;
        self.players[index].hand.add_card(
            self.deck.cards.remove(
                self.deck
                    .cards
                    .binary_search(&card::Card::new(card::Suit::Hearts, card::Rank::Three))
                    .expect("REASON"),
            ),
        );

        let first_player_name = &self.players[index].name;
        display::announce(format!("{first_player_name} will start first."));

        // Ensure first player is last when distributing cards to balance hand size
        for _ in index..self.players.len() - 1 {
            let popped_player = self
                .players
                .pop()
                .expect("Should always be at least one player");
            self.players.insert(0, popped_player);
        }

        // deal the rest of the cards
        while !self.deck.is_empty() {
            for player in &mut self.players {
                if self.deck.is_empty() {
                    break;
                }
                player.hand.add_card(self.deck.draw_card());
                player.hand.sort();
            }
        }

        //After dealing, put first_player in the first position
        let popped_player = self
            .players
            .pop()
            .expect("Should always be at least one player");
        self.players.insert(0, popped_player);
    }

    pub fn player_moves(&mut self) {
        let mut round = round::Round::new(); //Initialize new round
        for player in &mut self.players {
            //TODO Remember to change order of players before playing and loop this jawn until win
            display::announce_top_left(format!("Current Player: {0}", player.name), 0);
            display::announce_top_left(format!("Current Move Class: "), 1);
            display::announce(format!("{0}'s turn.'", player.name));
            let hand_size = player.hand.cards.len();
            let mut selected = vec![false; hand_size]; // array to represent card selection
            let mut selector = 0usize; // cursor to create selection

            let mut current_play: Option<play::Class> = Some(play::Class::Invalid);
            let mut play_rank: Option<card::Rank>;

            display::show_hand(&player.hand, &selected, selector);

            // card selection to be inputted into play
            loop {
                let current_state = display::get_keystate();
                match current_state {
                    display::Input_States::Esc => {
                        break;
                    }
                    display::Input_States::Right => {
                        selector = (selector + 1) % player.hand.cards.len();
                    }
                    display::Input_States::Left => {
                        selector = ((selector as i16 - 1)
                            .rem_euclid(player.hand.cards.len() as i16))
                            as usize;
                    }
                    display::Input_States::Space => {
                        selected[selector] = !selected[selector];
                    }
                    display::Input_States::Enter => {
                        // Play selected play
                        if current_play != Some(play::Class::Invalid) {
                            let mut index = 0usize;

                            let mut player_move = play::Play::new(player.clone());

                            while index < player.hand.cards.len() {
                                if selected[index] {
                                    player_move.add_card(player.hand.cards.remove(index));
                                    selected.remove(index);
                                } else {
                                    index += 1;
                                }
                            }
                            round.plays.push(player_move);
                            break;
                        }
                    }
                    _ => {
                        continue;
                    }
                }

                // Gather currently selected cards
                let mut selected_cards = Vec::new();

                for i in 0..selected.len() {
                    if selected[i] {
                        selected_cards.push(player.hand.cards[i].clone());
                    }
                }

                // Display the player's hand
                display::show_hand(&player.hand, &selected, selector);

                // Show play state of selected cards
                current_play = Some(play::identify_class(&mut selected_cards));
                display::player_note(
                    format!("Current move: {:?}", current_play.as_ref().unwrap()),
                    1,
                );
                if current_play != Some(play::Class::Invalid) {
                    play_rank = Some(play::identify_rank(&mut selected_cards));
                    display::player_note(format!("Move Rank: {:?}", play_rank.unwrap()), 0);
                }
            }
        }
    }

    pub fn end_game(&mut self) {
        display::cleanup();
        println!("Goodbye!");
    }
}
