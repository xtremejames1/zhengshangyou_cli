use crate::card;
use crate::client::Client;
use crate::display::Display;
use crate::display::{self, Warning};
use crate::logger::Logger;
use crate::play;
use crate::player::Player;
use crate::player_client::PlayerClient;
use crate::round;
use crate::round::Round;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct GameClient {
    pub player: Player,
    pub client: Client,
    pub players: VecDeque<PlayerClient>,
    pub rounds: Vec<round::Round>,
    pub logger: Arc<Mutex<Logger>>,
    pub display: Display,
}

impl GameClient {
    pub fn new(
        player: Player,
        client: Client,
        players: VecDeque<PlayerClient>,
        logger: Arc<Mutex<Logger>>,
        display: Display,
    ) -> Self {
        let rounds = Vec::new();
        Self {
            player,
            client,
            players,
            rounds,
            logger,
            display,
        }
    }

    //TODO: Use input box as parameter maybe so that player can send start signal
    pub fn wait_for_start(&mut self) {
        loop {
            let incoming_message = self.client.read().unwrap();
            match incoming_message.as_str() {
                "s\0" => {
                    self.logger
                        .lock()
                        .unwrap()
                        .log("Game started", Duration::ZERO);
                    self.play();
                }
                _ => {}
            }
        }
    }

    pub fn play(&mut self) {
        loop {
            let incoming_message = self.client.read().unwrap();
            match incoming_message.as_str() {
                "r\0" => {
                    self.play_round();
                }
                _ => {}
            }
        }
    }

    pub fn play_round(&mut self) {
        let mut round = Round::new();
        loop {
            let message = self.client.read().unwrap();
            if message.len() < 4 {
                // Handle error: message too short
                continue;
            }
            let (id, message) = message.split_at(4);
            match id {
                "m" => {
                    self.play_move(&mut round);
                }
                "p" => round.add_play(message.to_string().into()),
                "e" => break,
                _ => {}
            }
        }

        let winner = &round.plays.last().unwrap().player;
        let winner_name = &winner.name;
        self.logger.lock().unwrap().log(
            format!("{winner_name} won the round. They will start the next round."),
            Duration::ZERO,
        );

        self.rounds.push(round);
    }

    pub fn play_move<T>(&mut self, mut round: T)
    where
        T: AsMut<Round>,
    {
        let round = round.as_mut();
        let hand_size = self.player.hand.cards.len();
        let mut selected = vec![false; hand_size]; // array to represent card selection
        let mut selector = 0usize; // cursor to create selection

        let mut current_play: Option<play::Class> = Some(play::Class::Invalid);
        let mut play_rank: Option<card::Rank> = Some(card::Rank::Three);

        display::show_hand(&self.player.hand, &selected, selector);

        display::show_play(round.plays.last());

        // card selection to be inputted into play
        loop {
            let current_state = display::get_keystate();
            match current_state {
                display::Input_States::Esc => {
                    break;
                }
                display::Input_States::Right => {
                    selector = (selector + 1) % self.player.hand.cards.len();
                }
                display::Input_States::Left => {
                    selector = ((selector as i16 - 1)
                        .rem_euclid(self.player.hand.cards.len() as i16))
                        as usize;
                }
                display::Input_States::Space => {
                    selected[selector] = !selected[selector];
                }
                display::Input_States::Enter => {
                    // Play selected play
                    if current_play == Some(play::Class::Invalid) {
                        self.display
                            .add_renderable(Arc::new(Mutex::new(Warning::new(
                                "Please make a valid move.",
                                Duration::new(5, 0),
                            ))))
                    }
                    // Allow user to play card if empty round, or valid move
                    else if round.plays.is_empty()
                        || (current_play == Some(round.plays.last().unwrap().class)
                            && play_rank.unwrap() as usize
                                > round.plays.last().unwrap().rank as usize)
                    {
                        let mut index = 0usize;

                        let mut move_cards = Vec::new();

                        while index < self.player.hand.cards.len() {
                            if selected[index] {
                                move_cards.push(self.player.hand.cards.remove(index));
                                selected.remove(index);
                            } else {
                                index += 1;
                            }
                        }
                        let mut player_move = play::Play::new(self.player.clone());
                        player_move.set_cards(move_cards);
                        self.client.send(player_move);

                        // TODO: Send the move back here
                        break;
                    } else {
                        self.display
                            .add_renderable(Arc::new(Mutex::new(Warning::new(
                                "Invalid move.",
                                Duration::new(5, 0),
                            ))))
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
                    selected_cards.push(self.player.hand.cards[i].clone());
                }
            }

            // Display the player's hand
            display::show_hand(&self.player.hand, &selected, selector);

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

    pub fn end_game(&mut self) {
        display::cleanup();
        println!("Goodbye!");
    }
}
