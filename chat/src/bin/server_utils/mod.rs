use chat::{Message, Parser, UserAddr, debug, error, info};
use chat::{rd_sock_to_string, wr_string_to_sock};
use std::result::Result::Ok;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, broadcast::Receiver as bd_reciver, broadcast::Sender as bd_sender};
use tokio::task::JoinSet;
use tokio::time::Duration;
use tracing::warn;
mod timeout;
use timeout::Timeout;
pub mod axum;
pub mod clientset;
use clientset::ClientSet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Port to host
    #[arg(short, long)]
    pub port: String,
}

pub struct Server {
    pub serv_addr: String,
    pub client_set: ClientSet,
    pub tx_rm_user: Sender<String>,
}

struct Client {}
impl Client {
    fn run(
        addr: String,
        tx: bd_sender<Message>,
        sock_read: OwnedReadHalf,
        sock_write: OwnedWriteHalf,
        client_set: ClientSet,
        rx_admin: Receiver<String>,
    ) {
        let mut set = JoinSet::new();
        let tx_cl = tx.clone();
        let addr_cl = addr.clone();

        let (tx_writer, rx_writer) = mpsc::channel(1204);
        set.spawn(Client::send_to_client(rx_writer, sock_write));
        set.spawn(Client::uesr_handler(sock_read, tx.clone(), addr.clone()));
        set.spawn(Client::handle_broadcast(
            tx.subscribe(),
            tx_writer.clone(),
            addr.clone(),
        ));
        set.spawn(Client::recive_msg_from_admin(rx_admin, tx_writer.clone()));

        tokio::spawn(async move {
            let _ = set.join_next().await;
            info!("User disconnected: {}", addr_cl);
            let mut mutex_gurad = client_set.set.lock().await;
            mutex_gurad.remove(&addr_cl);

            let _ = tx_cl.send(Message::UserLeft(UserAddr::new(addr_cl)));
        });
    }

    pub async fn send_to_client(
        mut rx: Receiver<String>,
        mut sock_write: tokio::net::tcp::OwnedWriteHalf,
    ) {
        while let Some(value) = rx.recv().await {
            match wr_string_to_sock(&mut sock_write, value).await {
                Ok(_) => {}
                Err(_) => {
                    error!("error writing to socket");
                    return;
                }
            }
        }
    }

    async fn uesr_handler(mut sock_read: OwnedReadHalf, tx: bd_sender<Message>, addr: String) {
        loop {
            let read_future = rd_sock_to_string(&mut sock_read);
            let timeout = Timeout::new(Duration::from_secs(15), read_future);

            match timeout.await {
                Ok(val) => {
                    let Ok(buf) = val else {
                        return;
                    };
                    let Ok(msg) = Message::from_json(buf) else {
                        return;
                    };
                    match msg {
                        Message::ChatMessage(mut ch_msg) => {
                            ch_msg.addr = addr.clone();
                            println!("{}: {}", ch_msg.addr, ch_msg.msg);
                            let _ = tx.send(Message::ChatMessage(ch_msg));
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    debug!("Timeout : {}", addr);
                    return;
                }
            }
        }
    }

    async fn handle_broadcast(
        mut rx: bd_reciver<Message>,
        tx_writer: Sender<String>,
        addr: String,
    ) {
        while let Ok(msg) = rx.recv().await {
            let should_send = match &msg {
                Message::ChatMessage(m) => m.addr != addr,
                Message::UserJoined(m_addr) | Message::UserLeft(m_addr) => m_addr.addr != addr,
                _ => true,
            };
            if should_send {
                if let Ok(json) = msg.to_json() {
                    let _ = tx_writer.send(json).await;
                } else {
                    println!("error serialize");
                }
            }
        }
    }

    async fn recive_msg_from_admin(mut rx_rm_user: Receiver<String>, tx_writer: Sender<String>) {
        while let Some(msg) = rx_rm_user.recv().await {
            let Ok(json) = Message::from_json(msg.clone()) else {
                warn!("Error deserialize");
                continue;
            };
            match json {
                Message::ChatMessage(_) => {
                    let _ = tx_writer.send(msg.clone()).await;
                }
                Message::Kick => {
                    return;
                }
                _ => {
                    continue;
                }
            };
        }
    }
}

impl Server {
    pub fn new(serv_addr: String, set: ClientSet) -> Self {
        Server {
            serv_addr: serv_addr,
            client_set: set,
            tx_rm_user: mpsc::channel(1024).0,
        }
    }
    pub async fn listen_for_connections(self) {
        let Ok(listener) = TcpListener::bind(self.serv_addr).await else {
            error!("Error binding to addr");
            return;
        };

        if let Ok(local_addr) = listener.local_addr() {
            debug!("Listening on {}", local_addr);
        }
        let tx = broadcast::channel::<Message>(1024).0;

        while let Ok((socket, addr)) = listener.accept().await {
            info!("New user connected: {addr}");
            let addr_str = addr.to_string();

            let client_set_cl = self.client_set.clone();
            let (tx_admin, rx_admin) = mpsc::channel(1024);

            let mut mutex_gurad = client_set_cl.set.lock().await;
            mutex_gurad.insert(addr_str.clone(), tx_admin);

            let _ = tx.send(Message::UserJoined(UserAddr::new(addr_str.clone())));
            let (sock_read, sock_write) = socket.into_split();
            Client::run(
                addr_str.clone(),
                tx.clone(),
                sock_read,
                sock_write,
                client_set_cl.clone(),
                rx_admin,
            );
        }
    }
}
