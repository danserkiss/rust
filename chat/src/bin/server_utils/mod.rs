use chat::{Message, Parser, UserAddr, debug, error, info};
use chat::{rd_sock_to_string, wr_string_to_sock};
use std::result::Result::Ok;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinSet;
use tokio::time::Duration;
mod timeout;
use timeout::Timeout;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Port to host
    #[arg(short, long)]
    pub port: String,
}

pub struct Server {
    pub serv_addr: String,
}

struct Client {
    set: JoinSet<()>,
    // addr: String,
}
impl Client {
    fn new() -> Self {
        Client {
            set: JoinSet::new(),
        }
    }
    async fn run(
        mut set: JoinSet<()>,
        addr: String,
        tx: Sender<Message>,
        sock_read: OwnedReadHalf,
        sock_write: OwnedWriteHalf,
    ) {
        let tx_cl = tx.clone();
        let tx_cl2 = tx.clone();
        let tx_cl3 = tx.clone();

        let addr_cl = addr.clone();
        let addr_cl2 = addr.clone();
        let addr_cl3 = addr.clone();

        set.spawn(async move {
            Client::uesr_handler(sock_read, tx_cl, addr_cl).await;
        });
        set.spawn(async move {
            Client::broadcast_msg(tx_cl2.subscribe(), sock_write, addr_cl2).await;
        });
        tokio::spawn(async move {
            if let Some(_) = set.join_next().await {
                info!("User disconnected: {}", addr_cl3);
                let _ = tx_cl3.send(Message::UserLeft(UserAddr::new(addr_cl3)));
                set.abort_all();
            }
        });
    }

    async fn uesr_handler(mut sock_read: OwnedReadHalf, tx: Sender<Message>, addr: String) {
        loop {
            let read_future = rd_sock_to_string(&mut sock_read);
            let timeout = Timeout::new(Duration::from_secs(10), read_future);

            match timeout.await {
                Ok(val) => {
                    if let Ok(buf) = val {
                        if let Ok(msg) = Message::from_json(buf) {
                            match msg {
                                Message::ChatMessage(mut ch_msg) => {
                                    ch_msg.addr = addr.clone();
                                    println!("{}: {}", ch_msg.addr, ch_msg.msg);
                                    let _ = tx.send(Message::ChatMessage(ch_msg));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }
    }
    async fn broadcast_msg(
        mut rx: Receiver<Message>,
        mut sock_write: OwnedWriteHalf,
        addr: String,
    ) {
        loop {
            if let Ok(msg) = rx.recv().await {
                let should_send = match &msg {
                    Message::ChatMessage(m) => m.addr != addr,
                    Message::UserJoined(m_addr) | Message::UserLeft(m_addr) => m_addr.addr != addr,
                    _ => true,
                };
                if should_send {
                    if let Ok(json) = msg.to_json() {
                        let _ = wr_string_to_sock(&mut sock_write, json).await;
                    } else {
                        println!("error serialize");
                    }
                }
            }
        }
    }
}

impl Server {
    pub async fn listen_for_connections(self) {
        if let Ok(listener) = TcpListener::bind(self.serv_addr).await {
            if let Ok(local_addr) = listener.local_addr() {
                debug!("Listening on {}", local_addr);
            }
            let tx = broadcast::channel::<Message>(1024).0;

            loop {
                if let Ok((socket, addr)) = listener.accept().await {
                    info!("New user connected: {addr}");
                    let _ = tx.send(Message::UserJoined(UserAddr::new(addr.to_string())));
                    let (sock_read, sock_write) = socket.into_split();
                    let cl = Client::new();
                    Client::run(cl.set, addr.to_string(), tx.clone(), sock_read, sock_write).await;
                } else {
                    error!("Error accepting connection");
                }
            }
        }
    }
}
