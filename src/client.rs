use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Client {
    tcp_tx: mpsc::Sender<String>,
    pub tcp_thread: thread::JoinHandle<()>,
}

impl Client {
    pub fn new(ip: IpAddr, name: String) -> Result<Self, &'static str> {
        let mut stream = TcpStream::connect((ip, 9141)).unwrap();
        stream.set_nodelay(true);
        stream
            .write_all(format!("name:{name}\0").as_bytes())
            .unwrap();
        stream.flush();

        stream.set_read_timeout(Some(Duration::from_secs(20)));

        let mut data: [u8; 8] = [0; 8];
        stream.read_exact(&mut data);
        let data = String::from_utf8(data.to_vec()).expect("Invalid Server Response");

        if data.contains("err") {
            return Err("Connection error");
        } else {
            println!("Server response: {}", data)
        }

        let (tcp_tx, tcp_rx) = mpsc::channel();

        let tcp_thread = thread::spawn(move || loop {
            loop {
                let message = if let Ok(data) = tcp_rx.try_recv() {
                    println!("Sent {}", data);
                    data
                } else {
                    "ok\0".to_string()
                };

                stream.write_all(message.as_bytes());
                stream.flush();
            }
        });

        Ok(Self { tcp_tx, tcp_thread })
    }

    pub fn send(&mut self, message: String) {
        self.tcp_tx.send(message);
    }
}
