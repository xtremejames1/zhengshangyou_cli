use crate::card;
use crate::deck;
use crate::display;
use crate::logger::Logger;
use crate::play;
use crate::player::Player;
use crate::round;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::time::Duration;

pub struct GameServer {
    pub players_streams: VecDeque<(Player, TcpStream)>,
    pub rounds: Vec<round::Round>,
    pub deck: deck::Deck,
    pub logger: Logger,
}

impl GameServer {
    pub fn new(
        players_streams: VecDeque<(Player, TcpStream)>,
        deck: deck::Deck,
        logger: Logger,
    ) -> Self {
        let rounds = Vec::new();
        Self {
            players_streams,
            rounds,
            deck,
            logger,
        }
    }

    pub fn start_game(&mut self) {
        self.deal_cards();
        loop {
            let winner = self.play_round();
            if !winner.is_none() {
                let winner_name = winner.unwrap().name;
                self.logger.log(
                    format!("The winner is {winner_name}. Congratulations"),
                    Duration::ZERO,
                );
                break;
            }
        }
    }

    pub fn deal_cards(&mut self) {
        // pick random player to start, give them a 3 of hearts as standard in the game
        let index = (rand::random::<f32>() * self.players_streams.len() as f32).floor() as usize;

        self.deck.sort();

        self.players_streams[index].0.hand.add_card(
            self.deck.cards.remove(
                self.deck
                    .cards
                    .binary_search(&card::Card::new(card::Suit::Hearts, card::Rank::Three))
                    .expect("REASON"),
            ),
        );

        let first_player_name = &self.players_streams[index].0.name;
        self.logger.log(
            format!("{first_player_name} will start first."),
            Duration::ZERO,
        );

        // Ensure first player is last when distributing cards to balance hand size
        for _ in index..self.players_streams.len() - 1 {
            let last_player = self
                .players_streams
                .pop_front()
                .expect("Should have at least one player");

            self.players_streams.insert(0, last_player);
        }

        // deal the rest of the cards
        while !self.deck.is_empty() {
            for (player, _) in &mut self.players_streams {
                if self.deck.is_empty() {
                    break;
                }
                player.hand.add_card(self.deck.draw_card());
                player.hand.sort();
            }
        }

        //After dealing, put first_player in the first position
        let last_player = self
            .players_streams
            .pop_front()
            .expect("Should have at least one player");

        self.players_streams.insert(0, last_player);
    }

    pub fn play_round(&mut self) -> Option<Player> {
        //Optionally return a winner
        let mut round = round::Round::new(); //Initialize new round
        let mut active_player: &Player; // Sets active player as first player in list.

        'round: loop {
            // loop until everybody skips
            for (player, stream) in &mut self.players_streams {
                active_player = &*player;
                // If everyone besides the last play has skipped their turn, end round
                if !round.plays.is_empty() && active_player == &round.plays.last().unwrap().player {
                    break 'round;
                }

                let hand_size = player.hand.cards.len();
                let mut selected = vec![false; hand_size]; // array to represent card selection
                let mut selector = 0usize; // cursor to create selection

                let mut current_play: Option<play::Class> = Some(play::Class::Invalid);
                let mut play_rank: Option<card::Rank> = Some(card::Rank::Three);

                display::show_hand(&player.hand, &selected, selector);

                display::show_play(round.plays.last());

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
                            if current_play == Some(play::Class::Invalid) {
                                display::warn("Please make a valid move.".to_string());
                            }
                            // Allow user to play card if empty round, or valid move
                            else if round.plays.is_empty()
                                || (current_play == Some(round.plays.last().unwrap().class)
                                    && play_rank.unwrap() as usize
                                        > round.plays.last().unwrap().rank as usize)
                            {
                                let mut index = 0usize;

                                // let mut player_move = play::Play::new(active_player.clone());
                                let mut move_cards = Vec::new();

                                while index < player.hand.cards.len() {
                                    if selected[index] {
                                        move_cards.push(player.hand.cards.remove(index));
                                        selected.remove(index);
                                    } else {
                                        index += 1;
                                    }
                                }
                                let mut player_move = play::Play::new(player.clone());
                                player_move.set_cards(move_cards);

                                round.add_play(player_move);
                                break;
                            } else {
                                display::warn("Invalid move.".to_string());
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

                // win condition
                if player.hand.cards.is_empty() {
                    return Some(player.clone());
                }
            }
        }

        let winner = &round.plays.last().unwrap().player;
        let winner_name = &winner.name;
        self.logger.log(
            format!("{winner_name} won the round. They will start the next round."),
            Duration::ZERO,
        );

        while self.players_streams.front().unwrap().0 != round.plays.last().unwrap().player {
            let last_player = self
                .players_streams
                .pop_front()
                .expect("Should have at least one player");

            self.players_streams.push_back(last_player);
        }

        self.rounds.push(round);
        None
    }

    pub fn end_game(&mut self) {
        display::cleanup();
        println!("Goodbye!");
    }
}
