use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::{display, player};

pub struct Server {
    pub players_streams: Arc<Mutex<Vec<(player::Player, TcpStream)>>>,
    pub listener_thread: Option<thread::JoinHandle<()>>,
    pub data_thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            players_streams: Arc::new(Mutex::new(Vec::new())),
            listener_thread: None,
            data_thread: None,
        }
    }

    //TODO implement player maximum and such
    pub fn accept_players(&mut self) {
        // Create listener to listen for any new connections
        let listener = TcpListener::bind("127.0.0.1:9141").unwrap();

        let (stop_tx, stop_rx) = mpsc::channel();

        let players_streams = Arc::clone(&self.players_streams);

        // Concurrently run thread in order to receive connections
        self.listener_thread = Some(thread::spawn(move || {
            'listener: for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let ip = stream.peer_addr().unwrap();

                // println!("incoming connection from {}", ip);
                display::announce_top_left(format!("Incoming connection from {ip}"), 0);

                if let Ok(stop) = stop_rx.try_recv() {
                    display::announce_top_left(format!("Stop message received"), 8);
                    if stop {
                        break;
                    }
                }

                display::announce_top_left(format!("No internal stop command"), 2);
                // println!("No internal stop command");

                let mut players = players_streams.lock().unwrap();

                display::announce_top_left(format!("Obtained players_streams lock"), 3);
                // println!("lock obtained");

                display::announce_top_left(format!("Validating player"), 1);
                // println!("validating player");

                if let Some(user_name) = validate_player(&stream) {
                    if players.iter().any(|p| p.0.name == user_name) {
                        display::announce_top_left(
                            format!("Player attempted with duplicate name {user_name}"),
                            1,
                        );
                        // println!("Player attempted with duplicate name {}", user_name);
                        stream.write(&format!("err:name").as_bytes());
                        stream.shutdown(std::net::Shutdown::Both);
                        continue 'listener;
                    }
                    stream.write(&format!("connected").as_bytes());

                    display::announce_top_left(
                        format!("Player {user_name} connected from {ip}"),
                        2,
                    );
                    // println!("Player connected");
                    let player = player::Player::new(user_name);
                    players.push((player, stream));
                }
            }
        }));

        let players_streams2 = Arc::clone(&self.players_streams);

        self.data_thread = Some(thread::spawn(move || loop {
            let players_streams = players_streams2.lock().unwrap();
            for player in &*players_streams {
                let mut buf_reader = BufReader::new(&player.1);

                let mut data = Vec::new();

                buf_reader.read_until(b'\0', &mut data);

                let data = String::from_utf8(data).expect("Invalid signal");

                if data.contains("stop") {
                    stop_tx.send(true);

                    //connect to itself in order to unblock listener loop
                    TcpStream::connect("127.0.0.1:9142");
                } else {
                    display::announce_top_left("Craziness".to_string(), 7);
                }
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
