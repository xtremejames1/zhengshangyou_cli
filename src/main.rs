pub mod card;
pub mod deck;
pub mod hand;
pub mod play;
pub mod player;

use std::num::IntErrorKind;

fn main() {
    let num_decks = input_u32("How many decks would you like to play with?".to_string(), "decks".to_string());
    let mut deck = deck::Deck::new(num_decks);

    let num_players = input_u32("How many players would you like to play with?".to_string(), "players".to_string());
    println!("Starting game with {} players and {} decks.", num_players, num_decks);

    let mut players = Vec::new();
    for n in 0u32..num_players {
        let mut name = String::new();

        println!("Player {}, enter your name :", n+1u32);
        std::io::stdin().read_line(&mut name).unwrap();

        players.push(player::Player::new(name.trim().to_string()));
    }

    println!("All players added.");

    // pick random player to start, give them a 3 of hearts as standard in the game
    let index = (rand::random::<f32>() * players.len() as f32).floor() as usize;
    players[index].hand.add_card(deck.cards.remove(deck.cards.binary_search(&card::Card::new(card::Suit::Hearts, card::Rank::Three)).expect("REASON")));

    println!("{} will start first.", players[index].name);

    // deal the rest of the cards
    while !deck.is_empty() {
        for player in &mut players {
            if deck.is_empty() { break; }
            player.hand.add_card(deck.draw_card());
        }
    }

    // list all hands
    for player in &mut players {
        player.hand.sort(); // sort the players hand
        println!("{}'s hand: ", player.name);
        for card in &player.hand.cards {
            print!("{} ",card);
        }
        println!();
    }
}

fn input_u32( prompt: String, subject: String ) -> u32 {
    let mut line = String::new();
    loop {
        println!("{}", prompt);
        std::io::stdin().read_line(&mut line).unwrap();
        if let Err(e) = line.trim().parse::<u32>() {
            match e.kind() {
                IntErrorKind::Empty => {
                    println!("Please enter a value.");
                }
                IntErrorKind::InvalidDigit => {
                    println!("Please enter a valid number of {}.", subject);
                }
                error => {
                    panic!("Unexpected error {error:?}")
                }
            }
        }
        else if line.trim().parse::<u32>().unwrap() == 0 {
            println!("Cannot play with zero {}.", subject);
        }
        else {
            break line.trim().parse().unwrap();
        }
        line.clear()
    }
}
