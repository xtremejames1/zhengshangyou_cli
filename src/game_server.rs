use crate::card;
use crate::deck;
use crate::display;
use crate::logger::Logger;
use crate::play;
use crate::play::Play;
use crate::player::Player;
use crate::round;
use crate::server::Server;
use std::collections::VecDeque;
use std::time::Duration;

pub struct GameServer {
    pub server: Server,
    pub players_streams: VecDeque<Player>,
    pub rounds: Vec<round::Round>,
    pub deck: deck::Deck,
    pub logger: Logger,
}

impl GameServer {
    pub fn new(
        server: Server,
        players_streams: VecDeque<Player>,
        deck: deck::Deck,
        logger: Logger,
    ) -> Self {
        let rounds = Vec::new();
        Self {
            server,
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

        self.players_streams[index].hand.add_card(
            self.deck.cards.remove(
                self.deck
                    .cards
                    .binary_search(&card::Card::new(card::Suit::Hearts, card::Rank::Three))
                    .expect("REASON"),
            ),
        );

        let first_player_name = &self.players_streams[index].name;
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
            for player in &mut self.players_streams {
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

        //TODO: Send players their cards, also send order of players
    }

    pub fn play_round(&mut self) -> Option<Player> {
        //Optionally return a winner
        let mut round = round::Round::new(); //Initialize new round
        self.server.send_all("r\0"); //Send new round to clients

        'round: loop {
            // loop until everybody skips
            for player in &self.players_streams {
                // If everyone besides the last play has skipped their turn, end round
                if !round.plays.is_empty() && &*player == &round.plays.last().unwrap().player {
                    break 'round;
                }

                // TODO: Add non panicking error handling here
                self.server.send("m\0", player);

                //wait for response here

                let play_str = self
                    .server
                    .read(player)
                    .expect("Invalid play sent by player");
                let play: Play = play_str.clone().into();

                round.plays.push(play);

                // win condition
                if player.hand.cards.is_empty() {
                    return Some(player.clone());
                }

                self.server.send_all(format!("p{play_str}\0"));
            }
        }

        let winner = &round.plays.last().unwrap().player;
        let winner_name = &winner.name;
        self.logger.log(
            format!("{winner_name} won the round. They will start the next round."),
            Duration::ZERO,
        );

        while *self.players_streams.front().unwrap() != round.plays.last().unwrap().player {
            let last_player = self
                .players_streams
                .pop_front()
                .expect("Should have at least one player");

            self.players_streams.push_back(last_player);
        }

        self.server.send_all(format!("e\0"));
        self.rounds.push(round);
        None
    }

    pub fn end_game(&mut self) {
        display::cleanup();
        println!("Goodbye!");
    }
}
