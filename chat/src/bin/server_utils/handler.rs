use crate::server_utils::{ClientSet, timeout::Timeout};
use chat::{Message, UserAddr, debug, error, info};
use chat::{rd_sock_to_string, wr_string_to_sock};
use std::result::Result::Ok;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast::Receiver as bd_reciver, broadcast::Sender as bd_sender};
use tokio::task::JoinSet;
use tokio::time::Duration;
use tracing::warn;

pub struct Handler {}
impl Handler {
    pub fn run(
        addr: String,
        tx: bd_sender<Message>,
        sock_read: OwnedReadHalf,
        sock_write: OwnedWriteHalf,
        mut client_set: ClientSet,
        rx_admin: Receiver<String>,
    ) {
        let mut set = JoinSet::new();
        let tx_cl = tx.clone();
        let addr_cl = addr.clone();

        let (tx_writer, rx_writer) = mpsc::channel(1204);
        set.spawn(Handler::send_to_client(rx_writer, sock_write));
        set.spawn(Handler::uesr_handler(sock_read, tx.clone(), addr.clone()));
        set.spawn(Handler::handle_broadcast(
            tx.subscribe(),
            tx_writer.clone(),
            addr.clone(),
        ));
        set.spawn(Handler::recive_msg_from_admin(rx_admin, tx_writer.clone()));

        tokio::spawn(async move {
            let _ = set.join_next().await;
            let _ = tx_cl.send(Message::UserLeft(UserAddr::new(addr_cl.clone())));
            client_set.remove_user(addr_cl.clone()).await;
            info!("User disconnected: {}", addr_cl);
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
