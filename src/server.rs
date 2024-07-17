use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::display::Display;
use crate::player;

pub struct Server {
    pub player_network: Arc<Mutex<Vec<(player::Player, TcpStream, Instant)>>>,
    pub listener_thread: Option<thread::JoinHandle<()>>,
    pub running: Arc<Mutex<bool>>,
    pub display: Arc<Mutex<Display>>,
}

impl Server {
    pub fn new() -> Self {
        let display = Arc::new(Mutex::new(Display::new()));
        Self {
            player_network: Arc::new(Mutex::new(Vec::new())),
            listener_thread: None,
            running: Arc::new(Mutex::new(true)),
            display,
        }
    }

    //TODO implement player maximum and such
    pub fn accept_players(&mut self) {
        // Create listener to listen for any new connections
        let listener = TcpListener::bind("127.0.0.1:9141").unwrap();
        listener.set_nonblocking(true).unwrap();

        let players_streams = Arc::clone(&self.player_network);
        let running = Arc::clone(&self.running);
        let display = Arc::clone(&self.display);

        // Concurrently run thread in order to receive connections
        self.listener_thread = Some(thread::spawn(move || {
            while *running.lock().unwrap() {
                let mut players = players_streams.lock().unwrap();
                let mut display = display.lock().unwrap();

                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        // println!("incoming connection from {}", addr);
                        display.log(
                            format!("Incoming connection from {addr}"),
                            Duration::new(0, 0),
                        );
                        if let Some(user_name) = validate_player(&stream) {
                            if players.iter().any(|p| p.0.name == user_name) {
                                display.log(
                                    format!("Player attempted with duplicate name {user_name}"),
                                    Duration::new(0, 0),
                                );
                                // println!("Player attempted with duplicate name {}", user_name);
                                stream.write(&format!("err:name").as_bytes());
                                stream.shutdown(std::net::Shutdown::Both);
                                continue;
                            }
                            stream.write(&format!("connected").as_bytes());

                            display.log(
                                format!("Player {user_name} connected from {addr}"),
                                Duration::new(0, 0),
                            );
                            // println!("Player connected");
                            let player = player::Player::new(user_name);
                            players.push((player, stream, Instant::now()));
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => eprint!("Error accepting connection: {}", e),
                }

                for player in players.iter_mut() {
                    let mut buf_reader = BufReader::new(&player.1);

                    let mut data = Vec::new();
                    if let Some(_) = buf_reader.read_until(b'\0', &mut data).ok() {
                        match std::str::from_utf8(&data) {
                            Ok("ok\0") => {
                                player.2 = Instant::now(); //Update last connected time
                            }
                            Ok("stop\0") => {
                                *running.lock().unwrap() = false;
                            }
                            _ => {}
                        }
                    }
                }

                // Remove inactive players
                players.retain(|(player, stream, last_active)| {
                    if last_active.elapsed().as_secs() > 20 {
                        let user_name = &player.name;
                        display.log(
                            format!("Removing player {user_name} due to inactivity."),
                            Duration::new(0, 0),
                        );
                        stream.shutdown(std::net::Shutdown::Both).ok(); // Gracefully close the connection
                        false // Remove this player
                    } else {
                        true // Keep this player
                    }
                });

                display.show_logs();
            }
        }));
    }
}

fn validate_player(stream: &TcpStream) -> Option<String> {
    let mut buf_reader = BufReader::new(stream);

    let mut data = Vec::new();
    buf_reader.read_until(b'\0', &mut data).ok()?;

    let data = String::from_utf8(data).expect("Invalid name");

    let index = data.find("name:")?;
    let username = data.split_at(index + 5).1.trim().to_string();

    Some(username)
}
