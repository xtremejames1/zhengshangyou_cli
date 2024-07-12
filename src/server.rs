use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::player;

pub struct Server {
    stop_tx: Option<mpsc::Sender<bool>>,
    pub players_streams: Arc<Mutex<Vec<(player::Player, TcpStream)>>>,
    pub listener_thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            stop_tx: None,
            players_streams: Arc::new(Mutex::new(Vec::new())),
            listener_thread: None,
        }
    }

    //TODO implement player maximum and such
    pub fn accept_players(&mut self) {
        // Create listener to listen for any new connections
        let listener = TcpListener::bind("127.0.0.1:9141").unwrap();

        let (stop_tx, stop_rx) = mpsc::channel();

        self.stop_tx = Some(stop_tx);

        let players_streams = Arc::clone(&self.players_streams);

        // Concurrently run thread in order to receive connections
        self.listener_thread = Some(thread::spawn(move || {
            'listener: for stream in listener.incoming() {
                let stream = stream.unwrap();
                let ip = stream.peer_addr().unwrap();

                // println!("incoming connection from {}", ip);

                if stop_rx.try_recv().is_ok() {
                    break;
                }

                let mut players = players_streams.lock().unwrap();

                if players
                    .iter()
                    .any(|p| check_stop(&p.1).is_some() && check_stop(&p.1).unwrap())
                {
                    break 'listener;
                }

                // println!("validating player...");

                if let Some(user_name) = validate_player(&stream) {
                    if players.iter().any(|p| p.0.name == user_name) {
                        // println!(
                        //     "player attempted to connect with duplicate name {}.",
                        //     user_name
                        // );
                        stream.shutdown(std::net::Shutdown::Both);
                        continue 'listener;
                    }

                    // println!("player {} connected from {}", user_name, ip);
                    let player = player::Player::new(user_name);
                    players.push((player, stream));
                }
            }
        }));
    }

    pub fn stop_accepting_players(&mut self) {
        // println!("Stopping accepting players.");
        // send stop signal to listener thread
        self.stop_tx.as_mut().unwrap().send(true);

        //connect to itself in order to unblock listener loop
        TcpStream::connect("127.0.0.1:9142");
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

fn check_stop(stream: &TcpStream) -> Option<bool> {
    let mut buf_reader = BufReader::new(stream);

    let mut data = Vec::new();
    buf_reader.read_until(b'\0', &mut data).ok()?;

    let data = String::from_utf8(data).expect("Invalid signal");

    Some(data.contains("gamestart"))
}
