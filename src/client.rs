use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpListener, TcpStream};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(ip: IpAddr) -> Self {
        let stream = TcpStream::connect((ip, 9141)).unwrap();
        stream.set_nodelay(true);
        Self { stream }
    }

    pub fn send(&mut self, message: String) {
        self.stream.write_all(message.as_bytes()).unwrap();
        self.stream.flush();
    }
}
