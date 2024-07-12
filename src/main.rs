pub mod card;
pub mod client;
pub mod deck;
pub mod display;
pub mod game;
pub mod hand;
pub mod play;
pub mod player;
pub mod round;
pub mod server;

use std::{collections::VecDeque, num::IntErrorKind, thread, time::Duration};

fn main() {
    // TODO change this to launch arg
    let network_type = input_u32(
        "Enter 1 for client, 2 for server".to_string(),
        "bruh".to_string(),
    );

    match network_type {
        2 => {
            let mut server = server::Server::new();
            server.accept_players();
            // thread::sleep(Duration::from_secs(30));
            // server.stop_accepting_players();
            if server.players_streams.is_empty() {
                println!("no players found on server");
            }
            for player in server.players_streams {
                let player_name = player.0.name;
                let player_ip = player.1.peer_addr().unwrap();
                println!("name: {}", player_name);
                println!("ip: {}", player_ip);
            }
        }
        1 => {
            let name = input_string("What is your name?".to_string());
            let ip_string = input_string("What IP would you like to connect to?".to_string());
            let mut client = client::Client::new(ip_string.parse().unwrap());
            client.send(format!("name:{name}"));

            // TODO for some reason this blocks the thing from starting
            // maybe cuz client goes out of scope?
            let start_game = input_u32("Enter 1 to start game".to_string(), "bruh".to_string());
            if start_game == 1 {
                client.send("gamestart:".to_string());
            }
        }
        _ => {}
    }

    let num_decks = input_u32(
        "How many decks would you like to play with?".to_string(),
        "decks".to_string(),
    );

    let deck = deck::Deck::new(num_decks);

    let num_players = input_u32(
        "How many players would you like to play with?".to_string(),
        "players".to_string(),
    );
    println!(
        "Starting game with {} players and {} decks.",
        num_players, num_decks
    );

    let mut players = VecDeque::new();
    for n in 0u32..num_players {
        let mut name = String::new();

        println!("Player {}, enter your name :", n + 1u32);
        std::io::stdin().read_line(&mut name).unwrap();

        players.push_back(player::Player::new(name.trim().to_string()));
    }

    println!("All players added.");

    let mut game = game::Game::new(players, deck);

    game.start_game();
    game.end_game();
}

fn input_u32(prompt: String, subject: String) -> u32 {
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
        } else if line.trim().parse::<u32>().unwrap() == 0 {
            println!("Cannot play with zero {}.", subject);
        } else {
            break line.trim().parse().unwrap();
        }
        line.clear()
    }
}
fn input_string(prompt: String) -> String {
    let mut line = String::new();
    loop {
        println!("{}", prompt);
        std::io::stdin().read_line(&mut line).unwrap();
        if line.is_empty() {
            println!("Input cannot be empty.");
        } else {
            break;
        }
    }
    line.trim().to_string()
}
