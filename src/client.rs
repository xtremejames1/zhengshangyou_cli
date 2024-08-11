use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::str::from_utf8;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Client {
    tcp_tx: mpsc::Sender<String>,
    pub tcp_thread: thread::JoinHandle<()>,
    stream: Arc<Mutex<TcpStream>>,
}

impl Client {
    pub fn new(ip: IpAddr, name: String) -> Result<Self, &'static str> {
        let stream = Arc::new(Mutex::new(TcpStream::connect((ip, 9141)).unwrap()));
        let mut tcp_stream = stream.lock().unwrap();

        tcp_stream.set_nodelay(true);
        tcp_stream
            .write_all(format!("name:{name}\0").as_bytes())
            .unwrap();
        tcp_stream.flush();

        tcp_stream.set_read_timeout(Some(Duration::from_secs(20)));

        let mut data: [u8; 8] = [0; 8];
        tcp_stream.read_exact(&mut data);
        let data = String::from_utf8(data.to_vec()).expect("Invalid Server Response");

        if data.contains("err") {
            return Err("Connection error");
        } else {
            println!("Server response: {}", data)
        }

        let (tcp_tx, tcp_rx) = mpsc::channel();

        let stream_send_thread = Arc::clone(&stream);

        let tcp_thread = thread::spawn(move || loop {
            let mut tcp_stream = stream_send_thread.lock().unwrap();
            loop {
                let message = if let Ok(data) = tcp_rx.try_recv() {
                    println!("Sent {}", data);
                    data
                } else {
                    "ok\0".to_string()
                };

                tcp_stream.write_all(message.as_bytes());
                tcp_stream.flush();
            }
        });

        Ok(Self {
            tcp_tx,
            tcp_thread,
            stream: Arc::clone(&stream),
        })
    }

    pub fn send<T>(&mut self, message: T)
    where
        T: Into<String>,
    {
        self.tcp_tx.send(message.into());
    }
    pub fn read(&mut self) -> Result<String, std::io::Error> {
        let stream = self.stream.lock().unwrap();
        let mut buf_reader = BufReader::new(&*stream);

        let mut data = Vec::new();
        buf_reader.read_until(b'\0', &mut data)?;
        Ok(from_utf8(&data).unwrap().to_string())
    }
}
