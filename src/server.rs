use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::player;

pub struct Server {
    stop_tx: Option<mpsc::Sender<bool>>,
    players_rx: Option<mpsc::Receiver<Vec<(player::Player, TcpStream)>>>,
    pub players_streams: Vec<(player::Player, TcpStream)>,
}

impl Server {
    pub fn new() -> Self {
        let stop_tx = None;
        let players_rx = None;
        let players = Vec::new();
        Self {
            stop_tx,
            players_rx,
            players_streams: players,
        }
    }

    pub fn accept_players(&mut self) {
        // Create listener to listen for any new connections
        let listener = TcpListener::bind("127.0.0.1:9141").unwrap();

        let (stop_tx, stop_rx) = mpsc::channel();
        let (players_tx, players_rx) = mpsc::channel();

        self.stop_tx = Some(stop_tx);
        self.players_rx = Some(players_rx);

        // Concurrently run thread in order to receive connections
        thread::spawn(move || {
            let mut players: Vec<(player::Player, TcpStream)> = Vec::new();
            let mut stop = false; // boolean for if client calls for game start

            'listener: for stream in listener.incoming() {
                let stream = stream.unwrap();
                let ip = stream.peer_addr().unwrap();

                println!("incoming connection from {}", ip);

                for player in &mut players {
                    let buf_reader = BufReader::new(&player.1);
                    let data: Vec<_> = buf_reader
                        .lines()
                        .map(|result| result.unwrap())
                        .take_while(|line| !line.is_empty())
                        .collect();
                    if data.last().expect("No string found").contains("gamestart:") {
                        stop = true;
                    }
                }

                // If it receives stop signal from main thread, then stop
                if stop_rx.try_iter().next() == Some(true) || stop {
                    players_tx.send(players);
                    break;
                }

                println!("validating player...");

                let user_name = validate_player(&stream).unwrap();

                for player in &players {
                    if user_name == player.0.name {
                        stream.shutdown(std::net::Shutdown::Both);
                        println!(
                            "player attempted to connect with duplicate name {}.",
                            user_name
                        );

                        // do not allow iteration to continue
                        continue 'listener;
                    }
                }

                println!("player {} connected from {}", user_name, ip);

                let player = player::Player::new(user_name);
                players.push((player, stream));
            }
        });
    }

    pub fn stop_accepting_players(&mut self) {
        println!("Stopping accepting players.");
        // send stop signal to listener thread
        self.stop_tx.as_mut().unwrap().send(true);

        //connect to itself in order to unblock listener loop
        TcpStream::connect("127.0.0.1:9142");

        // receive player streams from listener thread
        self.players_streams = self.players_rx.as_mut().unwrap().recv().unwrap();
    }
}

fn validate_player(stream: &TcpStream) -> Option<String> {
    let buf_reader = BufReader::new(stream);
    println!("Created buffer");
    let data: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Read data");

    let index = data.last().expect("No string found").find("name:");
    println!("found name");

    let username = data.last()?.split_at(index.unwrap() + 5).1.to_string();

    Some(username)
}
