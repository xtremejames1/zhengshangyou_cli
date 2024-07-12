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

use display::announce_top_left;

fn main() {
    // TODO change this to launch arg
    let network_type = input_u32(
        "Enter 1 for client, 2 for server".to_string(),
        "bruh".to_string(),
    );

    match network_type {
        2 => {
            println!("Server started...");
            display::init();
            display::announce("Zheng Shang-You Server".to_string());
            let mut server = server::Server::new();
            server.accept_players();

            let mut player_names: Vec<String> = Vec::new();
            let mut refresh = true;

            loop {
                if server.listener_thread.as_ref().unwrap().is_finished() {
                    break;
                }
                let players_streams = server.players_streams.lock().unwrap();

                if refresh {
                    display::show_server_status(&players_streams);
                    refresh = false;
                }

                //Refresh if different amount of names
                if player_names.len() != players_streams.len() {
                    refresh = true;
                    if player_names.len() < players_streams.len() {
                        for i in 0..players_streams.len() - player_names.len() {
                            player_names.push(
                                players_streams[players_streams.len() - 1 - i]
                                    .0
                                    .name
                                    .clone(),
                            );
                        }
                    } else {
                        for _ in 0..player_names.len() - players_streams.len() {
                            player_names.pop();
                        }
                    }
                }

                //Refresh if the names are different
                for i in 0..players_streams.len() {
                    if players_streams[i].0.name != player_names[i] {
                        player_names.push(players_streams[i].0.name.clone());
                        player_names.swap_remove(i);
                        announce_top_left("Players updated".to_string(), 0);
                        refresh = true;
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        }
        1 => {
            let mut name: String;
            loop {
                name = input_string("What is your name?".to_string());
                if !name.contains('/') {
                    break;
                }
                println!("Username cannot contain \"/\". Please try again.")
            }
            let ip_string = input_string("What IP would you like to connect to?".to_string());
            let mut client = client::Client::new(ip_string.parse().unwrap());
            client.send(format!("name:{name}\0"));

            let start_game = input_u32("Enter 1 to start game".to_string(), "bruh".to_string());
            if start_game == 1 {
                client.send("gamestart\0".to_string());
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
