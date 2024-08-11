pub mod card;
pub mod client;
pub mod deck;
pub mod display;
pub mod game;
pub mod game_client;
pub mod game_server;
pub mod hand;
pub mod logger;
pub mod play;
pub mod player;
pub mod round;
pub mod server;

use std::{
    collections::VecDeque,
    net::IpAddr,
    num::IntErrorKind,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use display::{announce_top_left, Display, InputBox, Renderable};
use logger::Logger;

fn main() {
    // TODO change this to launch arg
    let network_type = input_u32(
        "Enter 1 for client, 2 for server".to_string(),
        "bruh".to_string(),
    );

    match network_type {
        2 => {
            println!("Server started...");
            let mut display = Display::new();
            let logger: Arc<Mutex<dyn Renderable>> = Arc::new(Mutex::new(Logger::new()));
            display.add_renderable(Arc::clone(&logger));
            let mut server = server::Server::new(logger);
            server.accept_players();

            let mut player_names: Vec<String> = Vec::new();
            let mut refresh = true;

            let start_time = Instant::now();

            loop {
                if server.listener_thread.as_ref().unwrap().is_finished() {
                    display::announce("Starting game.".to_string());
                    break;
                }
                let players_streams = server.player_network.lock().unwrap();

                if refresh {
                    display::show_server_status(&players_streams);
                    display.update();
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

                // Refresh every second
                if start_time.elapsed().as_millis() % 1000 < 120 {
                    refresh = true;
                }

                drop(players_streams);

                thread::sleep(Duration::from_millis(100));
            }
            let players_streams = server.player_network.lock().unwrap();
            let deck = deck::Deck::new((players_streams.len() / 4 + 1).try_into().unwrap());
        }
        1 => {
            let mut client: Result<client::Client, &'static str>;
            let mut display = display::Display::new();
            let logger: Arc<Mutex<dyn Renderable>> = Arc::new(Mutex::new(Logger::new()));

            display.add_renderable(Arc::clone(&logger));
            loop {
                let mut name: String;
                loop {
                    let name_input: Arc<Mutex<dyn Renderable>> =
                        Arc::new(Mutex::new(display::InputBox::new("Name:")));
                    display.add_renderable(Arc::clone(&name_input));

                    name = loop {
                        display.update();
                        if let Some(name) = &name_input
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_ref::<InputBox>()
                            .unwrap()
                            .output
                        {
                            break name.to_string();
                        }
                    };

                    if !name.contains('/') {
                        logger
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_mut::<Logger>()
                            .unwrap()
                            .log(format!("Welcome {name}"), Duration::new(5, 0));

                        display.update();
                        break;
                    }
                    logger
                        .lock()
                        .unwrap()
                        .as_any()
                        .downcast_mut::<Logger>()
                        .unwrap()
                        .log("Invalid name, please try again", Duration::new(5, 0));

                    display.update();
                }

                let mut ip_string: String;
                loop {
                    let ip_input: Arc<Mutex<dyn Renderable>> =
                        Arc::new(Mutex::new(display::InputBox::new("Server IP:")));
                    display.add_renderable(Arc::clone(&ip_input));
                    ip_string = loop {
                        display.update();
                        if let Some(name) = &ip_input
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_ref::<InputBox>()
                            .unwrap()
                            .output
                        {
                            break name.to_string();
                        }
                    };
                    if ip_string.parse::<IpAddr>().is_ok() {
                        logger
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_mut::<Logger>()
                            .unwrap()
                            .log(
                                format!("Attempting to connect to {ip_string}"),
                                Duration::new(5, 0),
                            );

                        display.update();
                        break;
                    }
                    logger
                        .lock()
                        .unwrap()
                        .as_any()
                        .downcast_mut::<Logger>()
                        .unwrap()
                        .log("Invalid IP, please try again", Duration::new(5, 0));

                    display.update();
                }
                client = client::Client::new(ip_string.parse().unwrap(), name.clone());
                match client {
                    Err(e) => {
                        logger
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_mut::<Logger>()
                            .unwrap()
                            .log(format!("Connection failed. {}", e), Duration::new(10, 0));
                    }
                    _ => {
                        logger
                            .lock()
                            .unwrap()
                            .as_any()
                            .downcast_mut::<Logger>()
                            .unwrap()
                            .log(format!("Connection Successful"), Duration::new(10, 0));
                        break;
                    }
                }
            }

            // Shadow client with actual client since not errored.
            let mut client = client.unwrap();

            // let start_game = input_u32("Enter 1 to start game".to_string(), "bruh".to_string());
            // if start_game == 1 {
            //     client.send("stop\0".to_string());
            // }
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
