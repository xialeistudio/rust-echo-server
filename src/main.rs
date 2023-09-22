use std::error::Error;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;
use bytes::Bytes;

#[derive(Debug)]
pub enum Event {
    Connect(Arc<Client>),
    Disconnect(Arc<Client>),
    Packet(Arc<Client>, Bytes),
}

/// 主线程负责从channel接受事件
/// 子线程1负责接收链接
/// 其他子线程进行网络收发
fn main() {
    let (main_tx, main_rx) = mpsc::channel();
    let main_tx_clone = main_tx.clone();
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:8484").unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let (client_tx, client_rx) = mpsc::channel();
            let client = Arc::new(Client::new(&stream, client_tx));
            let main_tx_clone = main_tx_clone.clone();

            let reader_stream = stream.try_clone().unwrap();
            thread::spawn(move || reader(reader_stream, client.clone(), main_tx_clone));
            thread::spawn(move || writer(stream, client_rx));
        }
    });
    // handle main events
    while let Ok(event) = main_rx.recv() {
        match event {
            Event::Connect(client) => {
                println!("{} connected", client.addr);
            }
            Event::Disconnect(client) => {
                println!("{} disconnected", client.addr);
            }
            Event::Packet(client, frame) => {
                client.send(frame);
            }
        }
    }
}

/// 发包线程
fn writer(mut stream: TcpStream, rx: Receiver<Bytes>) {
    while let Ok(frame) = rx.recv() {
        stream.write_all(&frame).unwrap();
    }
}

/// 读取数据
fn reader(mut stream: TcpStream, client: Arc<Client>, tx: Sender<Event>) {
    let _ = tx.send(Event::Connect(client.clone()));
    let mut buf = [0; 1024];
    loop {
        let n = match stream.read(&mut buf) {
            Ok(n) if n == 0 => break,
            Ok(n) => n,
            Err(_) => break
        };
        let frame = Bytes::from(Vec::from(&buf[0..n]));
        let _ = tx.send(Event::Packet(client.clone(), frame));
    }
    // 断开连接
    let _ = tx.send(Event::Disconnect(client.clone()));
}

#[derive(Debug)]
pub struct Client {
    addr: String,
    tx: Sender<Bytes>,
}

impl Client {
    pub fn new(stream: &TcpStream, tx: Sender<Bytes>) -> Client {
        Self {
            addr: stream.peer_addr().unwrap().to_string(),
            tx,
        }
    }

    pub fn send(&self, frame: Bytes) {
        self.tx.send(frame).unwrap();
    }
}