use std::process::exit;

use chat::{ChatMessage, Message, Parser, error, info, rd_sock_to_string, wr_string_to_sock};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinSet,
    time::Duration,
};
use tracing::warn;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // IP address and port to connect
    #[arg(short, long)]
    pub addr: String,
}

pub struct Client {
    pub addr: String,
}
impl Client {
    pub async fn connect(&self) {
        let Ok(stream) = TcpStream::connect(self.addr.clone()).await else {
            error!("cannot connect to server");
            return;
        };
        let (tx, rx) = mpsc::channel(1024);
        let (sock_read, sock_write) = stream.into_split();
        let tx_cl = tx.clone();

        let mut joinset = JoinSet::new();
        joinset.spawn(Client::send_to_server(rx, sock_write));
        joinset.spawn(Client::keep_alive(tx_cl));
        joinset.spawn(Client::read_msg_from_cli(tx.clone()));
        joinset.spawn(Client::recv_msg(sock_read));
        joinset.join_next().await;
        exit(0);
    }

    pub async fn send_to_server(
        mut rx: Receiver<String>,
        mut sock_write: tokio::net::tcp::OwnedWriteHalf,
    ) {
        while let Some(value) = rx.recv().await {
            match wr_string_to_sock(&mut sock_write, value).await {
                Ok(_) => {}
                Err(_) => {
                    error!("erro writing to socket");
                    return;
                }
            }
        }
    }

    async fn keep_alive(tx: Sender<String>) {
        while let Ok(json) = Message::keep_alive().to_json() {
            let _ = tx.send(json).await;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn read_msg_from_cli(tx: Sender<String>) {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();
        loop {
            input.clear();
            match reader.read_line(&mut input).await {
                Ok(_) => {
                    let trim_input = input.trim();
                    let msg = Message::ChatMessage(ChatMessage::new(
                        "".to_string(),
                        trim_input.to_string(),
                    ));
                    if let Ok(json) = msg.to_json() {
                        let _ = tx.send(json).await;
                    }
                }
                Err(_) => {
                    error!("Error reading from cli");
                    return;
                }
            }
        }
    }
    async fn recv_msg(mut sock_read: tokio::net::tcp::OwnedReadHalf) {
        while let Ok(buf) = rd_sock_to_string(&mut sock_read).await {
            let Ok(msg) = Message::from_json(buf) else {
                warn!("error deserialize");
                continue;
            };
            match msg {
                Message::ChatMessage(ch_str) => {
                    println!("{}: {}", ch_str.addr, ch_str.msg);
                }
                Message::UserJoined(addr) => {
                    println!("User connected: {}", addr.addr);
                }
                Message::UserLeft(addr) => {
                    println!("User disconnected: {}", addr.addr);
                }
                _ => {
                    continue;
                }
            }
        }
        info!("Disconnected");
        return;
    }
}
